import express from "express";
import db from "../db/index";
import { sendUpdateToUsers } from "./websocket";
import {
  getSession,
  getUser,
  checkUsername,
  getSessionUsers,
  uuid_regex,
  setDefaultPortfolio,
  insertNewUser,
} from "./utils";
import { startGamePhase } from "./plugins/start_game";

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
    const session_id = req.params.session_id;
    const session = await getSession(session_id);
    if (session === null)
      throw new CustomError(
        "Error, the session_id does not correspond to an existing session",
        404
      );
    if (session.status !== "open")
      throw new CustomError("Error, the session is not open for registration");
    const canInsertUsername = await checkUsername(session_id, username);
    if (!canInsertUsername)
      throw new CustomError(
        "Error, a user with this username is already registered to the session",
        409
      );

    // Insertion
    const user_id = await insertNewUser(session_id, username);
    res.status(201).json({ user_id: user_id });
    await setDefaultPortfolio(session_id, user_id);
    // Notify all users that a new user has joined
    notifyUsersListUpdate(session_id);
  } catch (error) {
    res.status(error.code).end(error.msg);
    return;
  }
}

/**
 * Get information about a given user
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getUserInfos(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const user_id = req.params.user_id;
    const user = await getUser(session_id, user_id);

    if (user === null)
      throw new CustomError("Error, cannot find an user with these IDs", 404);
    res.status(200).json({
      session_id: session_id,
      name: user.name,
      ready: user.game_ready,
    });
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
 * Mark a user as ready
 * @param req HTTP request
 * @param res HTTP response
 */
export async function setUserReady(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const user_id = req.params.user_id;
    const sessions = await getSession(session_id);
    if (sessions === null)
      throw new CustomError("Error, no session found with this ID", 404);
    if (sessions.status === "running")
      throw new CustomError("Error, the session is running");
    if (sessions.status === "closed")
      throw new CustomError("Error, the session is closed");

    const user = await getUser(session_id, user_id);
    if (user === null)
      throw new CustomError("Error, no user found with this ID");

    // Set user status to ready
    await db.query(
      "UPDATE users SET game_ready=TRUE WHERE session_id=$1 AND id=$2",
      [session_id, user_id]
    );
    res.status(201).end();

    // Notify all users that a user is ready
    notifyUsersListUpdate(session_id);

    // Check if the game can start
    startGamePhase(session_id);
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
 * Send the updated users list to connected users.
 * @param session_id ID of the auction
 */
async function notifyUsersListUpdate(session_id: string): Promise<void> {
  const users = await getSessionUsers(session_id);
  sendUpdateToUsers(
    session_id,
    "users_list_update",
    users.map((u) => {
      return { name: u.name, ready: u.game_ready };
    })
  );
}

const router = express.Router();

router.put(
  `/session/:session_id(${uuid_regex})/register_user`,
  registerNewUser
);
router.get(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})`,
  getUserInfos
);
router.put(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/ready`,
  setUserReady
);

export default router;
