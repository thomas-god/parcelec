import { v4 as uuidv4 } from "uuid";
import express from "express";
import db from "../db/index";
import { Session } from "./types";
import { getSession, getSessionUsers } from "./utils";
import { serialize } from "v8";

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
 * Get the list of sessions that are currently open.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getOpenSessions(
  req: express.Request,
  res: express.Response
): Promise<void> {
  const sessions: Session[] = (
    await db.query("SELECT id, name FROM sessions WHERE status='open'", [])
  ).rows;
  res.json(sessions);
}

/**
 * Open a new session with a user provided name. Name must be unique.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function openNewSession(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    const session_name: string = req.body.session_name;

    // Checks
    if (!session_name)
      throw new CustomError(
        "Error, please provide a valid game session name",
        400
      );
    if (
      (await db.query("SELECT id FROM sessions WHERE name=$1", [session_name]))
        .rows.length !== 0
    )
      throw new CustomError(
        "Error, a session already exists with this name",
        409
      );

    // Insertion
    const session: Session = {
      name: session_name,
      id: uuidv4(),
      status: "open",
    };
    await db.query(
      "INSERT INTO sessions (name, id, status) VALUES($1, $2, $3)",
      [session_name, session.id, session.status]
    );
    res.status(201).send(session);
  } catch (error) {
    res.status(error.code).end(error.msg);
    return;
  }
}

/**
 * Get informations for a specific session (status, step_no)
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getSessionInfos(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const session = await getSession(session_id);
    if (session === null)
      throw new CustomError(
        "Error, the session_id does not correspond to an existing session",
        404
      );
    const body: any = {
      id: session.id,
      name: session.name,
      status: session.status,
    };
    body.users = (await getSessionUsers(session_id))
      .map((user) => {
        return { name: user.name, ready: user.game_ready };
      })
      .sort((a, b) => (a.name > b.name ? 1 : -1));

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

const router = express.Router();

router.get("/sessions/open", getOpenSessions);
router.put("/session/", openNewSession);
router.get("/session/:session_id", getSessionInfos);

export default router;
