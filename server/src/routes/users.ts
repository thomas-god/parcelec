import express from 'express';
import db from '../db/index';
import { sendUpdateToUsers } from './websocket';
import {
  getSession,
  getUser,
  getSessionUsers,
  uuid_regex,
  CustomError,
} from './utils';
import { checkUserReadyAction } from './plugins/start_game';

// ---------------------- Routing Functions

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

    if (user === null) {
      throw new CustomError('Error, cannot find an user with these IDs', 404);
    }
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

// ---------------------- Helper Functions

/**
 * Send the updated users list to connected users.
 * @param session_id ID of the auction
 */
async function notifyUsersListUpdate(session_id: string): Promise<void> {
  const users = await getSessionUsers(session_id);
  sendUpdateToUsers(
    session_id,
    'users-list-update',
    users.map((u) => {
      return { name: u.name, ready: u.game_ready };
    })
  );
}

const router = express.Router();

router.get(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})`,
  getUserInfos
);

export default router;
