/**
 * Defines the routes for bids management.
 *  POST /session/:session_id/user/:user_id/bid
 *  GET /session/:session_id/user/:user_id/bids
 *  DELETE /session/:session_id/user/:user_id/bid/:bid_id
 *  GET /session/:session_id/clearing
 *  GET /session/:session_id/user/:user_id/clearing
 *
 */

import express from 'express';
import { v4 as uuid } from 'uuid';
import {
  getSession,
  getUser,
  uuid_regex,
  postBid,
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
 * Post a user bid to the current phase.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function postUserBid(
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
    if (session.status !== 'running') {
      throw new CustomError('Error, the session is not running');
    }

    // Payload checks
    if (req.body.bid === undefined) {
      throw new CustomError('Error, no bid payload provided');
    }
    if (req.body.bid.type === undefined) {
      throw new CustomError('Error, no bid type provided');
    }
    if (req.body.bid.type !== 'buy' && req.body.bid.type !== 'sell') {
      throw new CustomError('Error, no bid type must be `sell` or `buy`');
    }
    if (req.body.bid.volume_mwh === undefined) {
      throw new CustomError('Error, no bid volume_mwh provided');
    }
    if (
      req.body.bid.volume_mwh === '' ||
      isNaN(Number(req.body.bid.volume_mwh))
    ) {
      throw new CustomError(
        'Error, please provide a numeric value for the bid volume_mwh'
      );
    }
    if (req.body.bid.price_eur_per_mwh === undefined) {
      throw new CustomError('Error, no bid price_eur_per_mwh provided');
    }
    if (
      req.body.bid.price_eur_per_mwh === '' ||
      isNaN(Number(req.body.bid.price_eur_per_mwh))
    ) {
      throw new CustomError(
        'Error, please provide a numeric value for the bid price_eur_per_mwh'
      );
    }

    // Bid insertion
    const bid = {
      user_id: user_id,
      session_id: session_id,
      id: uuid(),
      type: req.body.bid.type,
      volume_mwh: Number(req.body.bid.volume_mwh),
      price_eur_per_mwh: Number(req.body.bid.price_eur_per_mwh),
    };
    await postBid(bid);
    res.status(201).json({ bid_id: bid.id });
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
