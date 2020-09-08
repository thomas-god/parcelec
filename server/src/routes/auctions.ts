import { v4 as uuidv4 } from "uuid";
import ws from "ws";
import url from "url";
import querystring from "querystring";
import express from "express";
import db from "../db/index";

class CustomError extends Error {
  msg: string;
  code: number;

  constructor(msg: string, code?: number, ...params) {
    super(...params);
    this.msg = msg;
    this.code = code || 400;
  }
}

export interface Auction {
  name: string;
  id: string;
  status: "Open" | "Running" | "Close";
}

export interface User {
  username: string;
  user_id: string;
}

export interface ClientMessage {
  username: string;
  date: Date;
  reason: "message" | "handshake";
  credentials: {
    auction_id: string;
    user_id: string;
  };
  data: any;
}

export interface UsersWebSocket {
  [index: string]: ws;
}

export interface AuctionsRecord {
  [index: string]: UsersWebSocket;
}

const auctions: AuctionsRecord = {};

// ---------------------- Routing Functions

/**
 * Get the list of auctions that are currently open.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getOpenAuctions(
  req: express.Request,
  res: express.Response
): Promise<void> {
  const auctions = (
    await db.query("SELECT id, name FROM auctions WHERE status='Open'", [])
  ).rows;
  res.send(JSON.stringify(auctions));
}

/**
 * Open a new auction with a user provided name. Name must be unique.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function openNewAuction(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    const auction_name: string = req.body.auction_name;

    // Checks
    if (!auction_name)
      throw new CustomError("Error, please provide a valid session name", 400);
    if (
      (await db.query("SELECT id FROM auctions WHERE name=$1", [auction_name]))
        .rows.length !== 0
    )
      throw new CustomError(
        "Error, a session already exists with this name",
        409
      );

    // Insertion
    const auction: Auction = {
      name: auction_name,
      id: uuidv4(),
      status: "Open",
    };
    await db.query(
      "INSERT INTO auctions (name, id, status) VALUES($1, $2, $3)",
      [auction_name, auction.id, "Open"]
    );
    res.status(201).send(auction);
  } catch (error) {
    res.status(error.code).end(error.msg);
    return;
  }
}

/**
 * Start a session (i.e. put its status to 'Running') provided its ID.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function startExistingAuction(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // Checks
    if (!req.body.auction_id) throw "Error, no auction ID provided";
    const auction = await getAuction(req.body.auction_id);
    if (!auction)
      throw "Error, the auction ID provided does not match an existing sessions";
    if (auction.status === "Running")
      throw "Error, the session is already running";
    if (auction.status === "Close") throw "Error, the session is closed";

    // Update
    await db.query("UPDATE auctions SET status=Running WHERE id=$1", [
      req.body.auction_id,
    ]);
  } catch (error) {
    res.status(400).end(error);
  }
}

/**
 * Add a user by its username to the list of user of an open session.
 * Username must be unique.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function registerNewUser(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // Payload checks
    if (!req.body.auction_id)
      throw new CustomError("Error, no auction_id provided", 400);
    if (!req.body.username)
      throw new CustomError("Error, no username provided", 400);

    // DB checks
    const auction = await getAuction(req.body.auction_id);
    if (!auction)
      throw new CustomError(
        "Error, the auction_id does not correspond to an existing auction",
        404
      );
    if (auction.status !== "Open")
      throw new CustomError(
        "Error, the auction is not open for registration",
        403
      );
    if (!checkUsername(req.body.auction_id, req.body.username))
      throw new CustomError("Error, the username already exist", 409);

    // Insertion
    const user_id = uuidv4();
    await db.query(
      "INSERT INTO users (id, auction_id, name) VALUES ($1, $2, $3)",
      [user_id, req.body.auction_id, req.body.username]
    );
    res.status(201).json({ user_id: user_id });
  } catch (error) {
    res.status(error.code).end(error.msg);
    return;
  }
}

/**
 * Callback when connecting a new WebSocket to check client's credentials
 * @param socket User WebSocket
 * @param request HTTP request
 */
export function onConnectionCallback(socket: ws, request: Request): void {
  try {
    const query_params = querystring.parse(url.parse(request.url).query);
    const auction_id = query_params.auction_id as string;
    const user_id = query_params.user_id as string;

    const user = getUser(auction_id, user_id);
    if (!user) throw "Error, connection not allowed";
    if (!auctions.hasOwnProperty(auction_id)) auctions[auction_id] = {};
    auctions[auction_id][user_id] = socket;
  } catch (error) {
    socket.terminate();
  }

  socket.on("message", (message: string) => {
    const msg = JSON.parse(message) as ClientMessage;
    msg.date = new Date();

    if (msg.reason === "message") {
      messageCallback(socket, msg);
    }
  });
}

/**
 * Handle new message and spread it to the connected users
 * @param socket Current WebSocket
 * @param msg Incoming message
 */
function messageCallback(socket: ws, msg: ClientMessage): void {
  try {
    // TODO: currently we do a DB call (via getUser) each time a message arrives
    // TODO: could we cache in some way the user info to avoid unnecessary DB calls ?
    // TODO: For instance by caching this info (e.g. in redis)
    const user = getUser(msg.credentials.auction_id, msg.credentials.user_id);
    if (!user) throw "Error, user not allowed";
    Object.values(auctions[msg.credentials.auction_id]).forEach((wss) => {
      wss.send(
        JSON.stringify({
          username: msg.username,
          date: msg.date,
          reason: "message",
          data: msg.data,
        })
      );
    });
  } catch (error) {
    socket.terminate();
  }
}

// ---------------------- Helper Functions

/**
 * Get an auction from the DB by its UUID
 * @param auction_id Auction UUID
 */
async function getAuction(auction_id: string): Promise<Auction> {
  const auction = (
    await db.query("SELECT * FROM auctions WHERE id=$1", [auction_id])
  ).rows;
  return auction.length === 1 ? auction[0] : null;
}

/**
 * Get the list of registered users to an auction from its UUID
 * @param auction_id Auction UUID
 */
async function getAuctionUsers(auction_id: string): Promise<User[]> {
  return (
    await db.query("SELECT * FROM users WHERE auction_id=$1", [auction_id])
  ).rows;
}

/**
 * Get a specific user registered to a specific auction
 * @param auction_id Auction UUID
 * @param user_id User UUID
 */
async function getUser(auction_id: string, user_id: string): Promise<User> {
  const user = (
    await db.query("SELECT * FROM users WHERE id=$1 AND auction_id=$2", [
      user_id,
      auction_id,
    ])
  ).rows;
  return user.length === 1 ? user[0] : null;
}

/**
 * Check if a given username can be registered to an auction (i.e. is
 * not already registered)
 * @param auction_id Auction UUID
 * @param username Username to be registered
 */
async function checkUsername(
  auction_id: string,
  username: string
): Promise<Boolean> {
  const users = (
    await db.query("SELECT * FROM users WHERE name=$1 AND auction_id=$2", [
      username,
      auction_id,
    ])
  ).rows;
  return users.length > 0;
}

const router = express.Router();

router.get("/list_open", getOpenAuctions);
router.put("/open", openNewAuction);
router.put("/register_user", registerNewUser);
router.put("/start", startExistingAuction);

export default router;
