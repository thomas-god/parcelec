import url from "url";
import querystring from "querystring";
import ws from "ws";
import { getUser } from "./utils";
import { ClientMessage } from "./types";

export interface UsersWebSocket {
  [index: string]: ws;
}

export interface SessionsRecord {
  [index: string]: UsersWebSocket;
}

const sessions: SessionsRecord = {};

/**
 * Callback when connecting a new WebSocket to check client's credentials
 * @param socket User WebSocket
 * @param request HTTP request
 */
export async function onConnectionCallback(
  socket: ws,
  request: Request
): Promise<void> {
  try {
    const query_params = querystring.parse(url.parse(request.url).query);
    const auction_id = query_params.auction_id as string;
    const user_id = query_params.user_id as string;

    const user = await getUser(auction_id, user_id);
    if (user === null) {
      throw "Error, connection not allowed";
    }
    if (!sessions.hasOwnProperty(auction_id)) {
      sessions[auction_id] = {};
    }
    sessions[auction_id][user_id] = socket;
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
    const user = getUser(msg.credentials.session_id, msg.credentials.user_id);
    if (!user) {
      throw "Error, user not allowed";
    }
    Object.values(sessions[msg.credentials.session_id]).forEach((wss) => {
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
 * @param session_id ID of the auction
 * @param reason Reason of the message
 * @param payload Content of the message
 */
export function sendUpdateToUsers(
  session_id: string,
  reason: string,
  payload: any
): void {
  if (Object.keys(sessions).includes(session_id)) {
    Object.values(sessions[session_id]).forEach((wss) => {
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
}

/**
 * Send an update/message to a specific user of the session, authored
 * as the SERVER.
 * @param session_id Session UUID
 * @param user_id User UUID
 * @param reason Reason of the message
 * @param payload Content of the message
 */
export function notifyUser(
  session_id: string,
  user_id: string,
  reason: string,
  payload: any
): void {
  if (Object.keys(sessions).includes(session_id)) {
    if (Object.keys(sessions[session_id]).includes(user_id)) {
      sessions[session_id][user_id].send(
        JSON.stringify({
          username: "SERVER",
          date: new Date(),
          reason: reason,
          data: payload,
        })
      );
    }
  }
}
