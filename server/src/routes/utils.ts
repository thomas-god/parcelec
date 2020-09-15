import db from "../db/index";
import { Session, User, Bid } from "./types";

export const uuid_regex = /^[A-F\d]{8}-[A-F\d]{4}-4[A-F\d]{3}-[89AB][A-F\d]{3}-[A-F\d]{12}$/i;

/**
 * Get a session from the DB by its UUID. Returns `null` if no
 * session is found.
 * @param session_id Session UUID
 */
export async function getSession(session_id: string): Promise<Session> {
  const session: Session[] = (
    await db.query("SELECT * FROM sessions WHERE id=$1", [session_id])
  ).rows;
  return session.length === 1 ? session[0] : null;
}

/**
 * Get the list of registered users to a session. Returns an empty list if
 * no users are found.
 * @param session_id Session UUID
 */
export async function getSessionUsers(session_id: string): Promise<User[]> {
  return (
    await db.query("SELECT * FROM users WHERE session_id=$1", [session_id])
  ).rows;
}

/**
 * Get a user Object. Returns `null` if it's not found.
 * @param session_id Session UUID
 * @param user_id User UUID
 */
export async function getUser(
  session_id: string,
  user_id: string
): Promise<User> {
  const user = (
    await db.query("SELECT * FROM users WHERE id=$1 AND session_id=$2", [
      user_id,
      session_id,
    ])
  ).rows;
  return user.length === 1 ? user[0] : null;
}

/**
 * Check if a given username can be registered to an session (i.e. is
 * not already registered). Return `true` if the user can be inserted
 * with this username.
 * @param session_id Session UUID
 * @param username Username to be registered
 */
export async function checkUsername(
  session_id: string,
  username: string
): Promise<boolean> {
  const users = (
    await db.query("SELECT * FROM users WHERE name=$1 AND session_id=$2", [
      username,
      session_id,
    ])
  ).rows;
  return users.length === 0;
}

/**
 * Get the number of the active step (i.e. with an 'open' status)
 * @param auction_id ID of the auction
 */
export async function getAuctionCurrentStep(
  auction_id: string
): Promise<number> {
  const res = (
    await db.query(
      "SELECT step_no FROM auctions_steps WHERE auction_id=$1 AND status='open'",
      [auction_id]
    )
  ).rows;
  return res.length === 1 ? (res[0].step_no as number) : null;
}

/**
 * Get a user's bid for the active step of an auction. Return null if the
 * user has not bid yet.
 * @param auction_id ID of the auction
 * @param user_id ID of the user
 */
export async function getBid(
  auction_id: string,
  user_id: string
): Promise<Bid> {
  const step_no = await getAuctionCurrentStep(auction_id);
  const res = (
    await db.query(
      "SELECT * FROM bids WHERE auction_id=$1 AND user_id=$2 AND auction_step_no=$3",
      [auction_id, user_id, step_no]
    )
  ).rows;
  return res.length === 1 ? (res[0] as Bid) : null;
}

/**
 * Return all the bids for the active step of an auction
 * @param auction_id ID of the auction
 */
export async function getAllBids(auction_id: string): Promise<Bid[]> {
  const step_no = await getAuctionCurrentStep(auction_id);
  const res = (
    await db.query(
      "SELECT * FROM bids WHERE auction_id=$1 AND auction_step_no=$2",
      [auction_id, step_no]
    )
  ).rows as Bid[];
  return res.length > 0 ? res : null;
}

/**
 * Return true if the user can bid and false if it can't (auction not running
 * or has already bid).
 * @param auction_id Auction ID
 * @param user_id User ID
 */
export async function checkUserCanBid(
  auction_id: string,
  user_id: string
): Promise<boolean> {
  const user = await getUser(auction_id, user_id);
  if (user === null) return false;

  const auction = await getSession(auction_id);
  if (auction.status !== "running") return false;

  const bid = await getBid(auction_id, user_id);
  return bid === null ? true : false;
}
