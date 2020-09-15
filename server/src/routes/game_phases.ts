/**
 * Defines the routes for the various game phases.
 */

import express from "express";
import { getSession, getUser, getPortfolio, uuid_regex } from "./utils";

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
export async function getUserPortfolio(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    console.log("getting port");
    // DB checks
    const session_id = req.params.session_id;
    const user_id = req.params.user_id;
    const session = await getSession(session_id);
    if (session === null)
      throw new CustomError("Error, no session found with this ID", 404);
    const user = await getUser(session_id, user_id);
    if (user === null)
      throw new CustomError("Error, no user found with this ID", 404);

    const portfolio = await getPortfolio(user_id);
    res.json(portfolio);
  } catch (error) {
    if (error instanceof CustomError) {
      res.status(error.code).end(error.msg);
    } else {
      res.status(400).end();
      throw error;
    }
  }
}

const router = express.Router();

router.get(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/portfolio`,
  getUserPortfolio
);

export default router;
