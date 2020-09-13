import db from "../db/index";
import { Auction, User, Bid } from "./types";

/**
 * Get an auction from the DB by its UUID
 * @param auction_id Auction UUID
 */
export async function getAuction(auction_id: string): Promise<Auction> {
  const auction = (
    await db.query("SELECT * FROM auctions WHERE id=$1", [auction_id])
  ).rows;
  return auction.length === 1 ? auction[0] : null;
}

/**
 * Get the list of registered users to an auction from its UUID
 * @param auction_id Auction UUID
 */
export async function getAuctionUsers(auction_id: string): Promise<User[]> {
  return (
    await db.query("SELECT * FROM users WHERE auction_id=$1", [auction_id])
  ).rows;
}

/**
 * Get a specific user registered to a specific auction
 * @param auction_id Auction UUID
 * @param user_id User UUID
 */
export async function getUser(
  auction_id: string,
  user_id: string
): Promise<User> {
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
 * not already registered). Return `true` if the user can be inserted
 * with this username.
 * @param auction_id Auction UUID
 * @param username Username to be registered
 */
export async function checkUsername(
  auction_id: string,
  username: string
): Promise<Boolean> {
  const users = (
    await db.query("SELECT * FROM users WHERE name=$1 AND auction_id=$2", [
      username,
      auction_id,
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
