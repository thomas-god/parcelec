/**
 * Defines the routes for bids management.
 *  GET /session/:session_id/user/:user_id/bids
 *  DELETE /session/:session_id/user/:user_id/bid/:bid_id
 *  GET /session/:session_id/clearing
 *  GET /session/:session_id/user/:user_id/clearing
 *
 */

import express from 'express';
import {
  getSession,
  getUser,
  uuid_regex,
  getUserBids,
  getUserBid,
  deleteUserBid,
  getClearing,
  getUserEnergyExchanges,
  CustomError,
  getClearedPhaseBids,
} from './utils';

// ---------------------- Routing Functions

/**
 * Return the list of a user's bids for the current phase.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getUserBidsRoute(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const user_id = req.params.user_id;
    const session = await getSession(session_id);
    if (session === null) {
      throw new CustomError('Error, no session found with this ID', 404);
    }
    const user = await getUser(session_id, user_id);
    if (user === null) {
      throw new CustomError('Error, no user found with this ID', 404);
    }

    // Getting the bids from the DB
    const bids = await getUserBids(session_id, user_id);
    res.status(200).json(bids);
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
 * Delete a given bid from the current phase.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function deleteUserBidRoute(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const user_id = req.params.user_id;
    const bid_id = req.params.bid_id;
    const session = await getSession(session_id);
    if (session === null) {
      throw new CustomError('Error, no session found with this ID', 404);
    }
    const user = await getUser(session_id, user_id);
    if (user === null) {
      throw new CustomError('Error, no user found with this ID', 404);
    }
    const bid = await getUserBid(session_id, bid_id);
    if (bid === null) {
      throw new CustomError('Error, no bid found with this ID', 404);
    }

    // Deleting the bids
    await deleteUserBid(session_id, bid_id);
    res.status(200).end();
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
 * Get the clearing information for the current phase.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getClearingRoute(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const session = await getSession(session_id);
    if (session === null) {
      throw new CustomError('Error, no session found with this ID', 404);
    }

    const clearing = await getClearing(session_id);
    res.status(200).json(clearing);
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
 * Get the user's energy exchanges following bids clearing.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getUserEnergyExchangesRoute(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const user_id = req.params.user_id;
    const session = await getSession(session_id);

    // Session exists and is running
    if (session === null) {
      throw new CustomError('Error, no session found with this ID', 404);
    }

    // User exists
    const user = await getUser(session_id, user_id);
    if (user === null) {
      throw new CustomError('Error, no user found with this ID', 404);
    }

    // Getting and sending the exchanges
    const exchanges = await getUserEnergyExchanges(session_id, user_id);
    res.status(200).json(exchanges);
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
 * Get the clearing information for the current phase.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getClearingAllBids(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const session = await getSession(session_id);
    if (session === null) {
      throw new CustomError('Error, no session found with this ID', 404);
    }

    const user_id = req.query.user_id as string;
    const bids = await getClearedPhaseBids(session_id, user_id);
    res.status(200).json(bids);
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

router.post(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/bid`,
  postUserBid
);
router.get(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/bids`,
  getUserBidsRoute
);
router.delete(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/bid/:bid_id(${uuid_regex})`,
  deleteUserBidRoute
);
router.get(`/session/:session_id(${uuid_regex})/clearing`, getClearingRoute);
router.get(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/clearing`,
  getUserEnergyExchangesRoute
);
router.get(
  `/session/:session_id(${uuid_regex})/clearing/all_bids`,
  getClearingAllBids
);

export default router;
