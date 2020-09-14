import { v4 as uuidv4 } from "uuid";
import express from "express";
import db from "../db/index";
import { sendUpdateToAuctionUsers } from "./websocket";
import { Auction, Bid } from "./types";
import {
  getAuction,
  getUser,
  checkUsername,
  getBid,
  getAllBids,
  getAuctionCurrentStep,
  getAuctionUsers,
} from "./utils";

class CustomError extends Error {
  msg: string;
  code: number;

  constructor(msg: string, code?: number, ...params) {
    super(...params);
    this.msg = msg;
    this.code = code || 400;
  }
}

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
 * Get informations for a specific auction (status, step_no)
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getAuctionInfos(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const auction_id = req.params.auction_id;
    const auction = await getAuction(auction_id);
    if (!auction)
      throw new CustomError(
        "Error, the auction_id does not correspond to an existing auction",
        404
      );
    const body: any = {
      id: auction.id,
      name: auction.name,
      status: auction.status,
    };
    body.users = (await getAuctionUsers(auction_id))
      .map((user) => {
        return { name: user.name, ready: user.ready };
      })
      .sort((a, b) => (a.name > b.name ? 1 : -1));

    if (auction.status === "Running") {
      body.step_no = (
        await db.query(
          "SELECT step_no FROM auctions_steps WHERE auction_id=$1 AND status='open'",
          [auction_id]
        )
      ).rows[0].step_no;
    }
    res.json(body);
  } catch (error) {
    if (error instanceof CustomError) {
      res.status(error.code).end(error.msg);
    } else {
      res.status(400).end();
      throw error;
    }
  }
}

/**
 * Register a new user by its username to an open session.
 * Username must be unique within the auction.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function registerNewUser(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // Payload checks
    if (req.body.username === null)
      throw new CustomError("Error, no username provided");
    const username = req.body.username;

    // DB checks
    const auction_id = req.params.auction_id;
    const auction = await getAuction(auction_id);
    if (auction === null)
      throw new CustomError(
        "Error, the auction_id does not correspond to an existing auction",
        404
      );
    if (auction.status !== "Open")
      throw new CustomError("Error, the auction is not open for registration");
    const canInsertUsername = await checkUsername(auction_id, username);
    if (!canInsertUsername)
      throw new CustomError(
        "Error, a user with this username is already registered to the auction",
        409
      );

    // Insertion
    const user_id = uuidv4();
    await db.query(
      "INSERT INTO users (id, auction_id, name) VALUES ($1, $2, $3)",
      [user_id, auction_id, username]
    );
    res.status(201).json({ user_id: user_id });
    // Notify all users that a new user has joined
    notifyUsersListUpdate(auction_id);
  } catch (error) {
    res.status(error.code).end(error.msg);
    return;
  }
}

/**
 * Mark a user as ready
 * @param req HTTP request
 * @param res HTTP response
 */
export async function setUserReady(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // Payload checks
    if (!req.body.user_id) throw new CustomError("Error, no user ID provided");
    const user_id = req.body.user_id;

    // DB checks
    const auction_id = req.params.auction_id;
    const auction = await getAuction(auction_id);
    if (auction === null)
      throw new CustomError("Error, no auction found with this ID", 404);
    if (auction.status === "Running")
      throw new CustomError("Error, the auction is running");
    if (auction.status === "Close")
      throw new CustomError("Error, the auction is closed");

    const user = await getUser(auction_id, user_id);
    if (user === null)
      throw new CustomError("Error, no user found with this ID");

    // Set user status to ready
    await db.query(
      "UPDATE users SET ready=TRUE WHERE auction_id=$1 AND id=$2",
      [auction_id, user_id]
    );
    res.status(201).end();

    // Notify all users that a user is ready
    notifyUsersListUpdate(auction_id);

    // Check if the auction can be started (i.e. set to status running)
    const users = await getAuctionUsers(auction_id);
    if (
      users.length >= 2 &&
      users.filter((u) => u.ready).length === users.length
    ) {
      await db.query("UPDATE auctions SET status='Running' WHERE id=$1", [
        auction_id,
      ]);
      await db.query(
        "INSERT INTO auctions_steps (auction_id, step_no, status) VALUES  ($1, $2, $3)",
        [auction_id, 0, "open"]
      );
      sendUpdateToAuctionUsers(auction_id, "auction_started", {});
    }
  } catch (error) {
    if (error instanceof CustomError) {
      res.status(error.code).end(error.msg);
    } else {
      res.status(400).end(error.message);
    }
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
    if (!req.params.auction_id)
      throw new CustomError("Error, no auction found with this ID");
    const auction_id = req.params.auction_id;
    const auction = await getAuction(auction_id);
    if (!auction)
      throw new CustomError(
        "Error, the auction ID does not match an existing auction",
        404
      );
    if (auction.status === "Running")
      throw new CustomError("Error, the auction is already running");
    if (auction.status === "Close")
      throw new CustomError("Error, the auction is closed");

    const users = await getAuctionUsers(auction_id);
    if (users.length < 2)
      throw new CustomError(
        "Error, not enough users registered to start the auction"
      );
    if (users.filter((u) => u.ready).length !== users.length)
      throw new CustomError(
        "Error, not all users are ready to start the auction"
      );

    // Update
    await db.query("UPDATE auctions SET status='Running' WHERE id=$1", [
      auction_id,
    ]);
    await db.query(
      "INSERT INTO auctions_steps (auction_id, step_no, status) VALUES ($1, $2, $3)",
      [auction_id, 0, "open"]
    );
    res.end();
  } catch (error) {
    res.status(error.code).end(error.msg);
  }
}

/**
 * Submit a bid to an open auction's step
 * @param req HTTP request
 * @param res HTTP response
 */
export async function submitBid(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // Check auction
    const auction_id = req.params.auction_id;
    const auction = await getAuction(auction_id);
    if (auction === null)
      throw new CustomError(
        "Error, the auction ID does not match an existing auction",
        404
      );
    if (auction.status !== "Running")
      throw new CustomError(
        "Error, the auction is not running and bids cannot be received"
      );

    // Check user
    if (!req.body.user_id) throw new CustomError("Error, no user_id specified");
    const user_id = req.body.user_id;
    const user = await getUser(auction_id, user_id);
    if (user === null)
      throw new CustomError("Error, no registered user found with this ID");

    // Check bid value
    if (!req.body.bid) throw new CustomError("Error, no bid value provided");

    // Check if the user has already bid
    const bid = await getBid(auction_id, user_id);
    if (bid !== null) throw new CustomError("Error, this user has already bid");

    // Insert bid
    const step_no = await getAuctionCurrentStep(auction_id);
    await db.query(
      "INSERT INTO bids (user_id, auction_id ,auction_step_no, bid_value) VALUES ($1, $2, $3, $4)",
      [user_id, auction_id, step_no, req.body.bid]
    );
    res.status(201).end();
  } catch (error) {
    res.status(error.code).end(error.msg);
  }
}

/**
 * Do the clearing of the current auction step
 * @param req HTTP request
 * @param res HTTP response
 */
export async function clearAuctionStep(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // Check auction
    const auction_id = req.params.auction_id;
    const auction = await getAuction(auction_id);
    if (auction === null)
      throw new CustomError("Error, cannot find an auction with this ID", 404);
    if (auction.status !== "Running")
      throw new CustomError(
        "Error, the auction is not running and cannot be cleared"
      );

    // Check bids
    const step_no = await getAuctionCurrentStep(auction_id);
    const bids = await getAllBids(auction_id);
    if (bids === null)
      throw new CustomError(
        "Error, this auction step does not contain any bids"
      );

    // Clear the auction
    const clearing_value = findMaxBid(bids);
    await db.query(
      "UPDATE auctions_steps SET status='closed', clearing_price=$1 WHERE auction_id=$2 AND step_no=$3",
      [clearing_value, auction_id, step_no]
    );
    await db.query(
      "INSERT INTO auctions_steps (auction_id, step_no, status) VALUES ($1, $2, $3)",
      [auction_id, step_no + 1, "closed"]
    );
    res.json({
      current_step_no: step_no,
      clearing_value: clearing_value,
      next_step_no: step_no + 1,
    });
  } catch (error) {
    if (error instanceof CustomError) {
      res.status(error.code).end(error.msg);
    } else {
      res.status(400).end(error.message);
    }
  }
}

// ---------------------- Helper Functions

/**
 * Find the largest bid of an auction step
 * @param bids Array of bids
 */
function findMaxBid(bids: Bid[]): number {
  if (bids.length === 0) return null;
  let max = bids[0].bid_value;
  for (let i = 0; i < bids.length; i++) {
    if (bids[i].bid_value > max) max = bids[i].bid_value;
  }
  return max;
}

/**
 * Send the updated users list to connected users.
 * @param auction_id ID of the auction
 */
async function notifyUsersListUpdate(auction_id: string): Promise<void> {
  const users = await getAuctionUsers(auction_id);
  sendUpdateToAuctionUsers(
    auction_id,
    "users_list_update",
    users.map((u) => {
      return { name: u.name, ready: u.ready };
    })
  );
}

const router = express.Router();

router.get("/list_open", getOpenAuctions);
router.put("/open", openNewAuction);
router.get("/:auction_id", getAuctionInfos);
router.put("/:auction_id/register_user", registerNewUser);
router.put("/:auction_id/user_ready", setUserReady);
router.put("/:auction_id/start", startExistingAuction);
router.put("/:auction_id/bid", submitBid);
router.put("/:auction_id/clearing", clearAuctionStep);

export default router;
