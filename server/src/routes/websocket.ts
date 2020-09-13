import ws from "ws";
import url from "url";
import querystring from "querystring";
import { getUser } from "./utils";
import { ClientMessage } from "./types";

export interface UsersWebSocket {
  [index: string]: ws;
}

export interface AuctionsRecord {
  [index: string]: UsersWebSocket;
}

const auctions: AuctionsRecord = {};

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

/**
 * Send an update/message to all connected users of an auction, authored
 * as the SERVER.
 * @param auction_id ID of the auction
 * @param reason Reason of the message
 * @param payload Content of the message
 */
export function sendUpdateToAuctionUsers(
  auction_id: string,
  reason: string,
  payload: any
): void {
  Object.values(auctions[auction_id]).forEach((wss) => {
    wss.send(
      JSON.stringify({
        username: "SERVER",
        date: new Date(),
        reason: reason,
        data: payload,
      })
    );
  });
}