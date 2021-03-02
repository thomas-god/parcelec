/**
 * Define the route for OTC energy exchanges between players.
 *  GET /session/:session_id/user/:user_id/otc
 *  POST /session/:session_id/user/:user_id/otc
 *  PUT /session/:session_id/user/:user_id/otc/:otc_id/accept
 *  PUT /session/:session_id/user/:user_id/otc/:otc_id/reject
 */
import express from 'express';
import { v4 as uuid } from 'uuid';
import db from '../db';
import { OTCEnergyExchange } from './types';
import {
  uuid_regex,
  CustomError,
  getSession,
  getUser,
  getLastPhaseInfos,
  checkUserInSessionByName,
  getUserOTCs,
  getOTCByID,
} from './utils';
import { notifyUser } from './websocket';

/**
 * Get list of OTCs involving the user (from or to).
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getUserOTCsRoute(
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

    // Getting the OTCs from the DB
    const otcs = await getUserOTCs(session_id, user_id);
    res.status(200).json(otcs);
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
 * Post a new OTC exchange.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function postOTC(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    const session_id = req.params.session_id;
    const user_id = req.params.user_id;
    const session = await getSession(session_id);

    /**
     * DB checks
     */
    if (session === null) {
      throw new CustomError('Error, no session found with this ID', 404);
    }
    const user = await getUser(session_id, user_id);
    if (user === null) {
      throw new CustomError('Error, no user found with this ID', 404);
    }
    const phase_infos = await getLastPhaseInfos(session_id);
    if (!phase_infos.plannings_allowed) {
      throw new CustomError('Error, cannot post new OTC exchange');
    }

    /**
     * Payload checks
     */
    if (
      req.body.type === undefined ||
      !['sell', 'buy'].includes(req.body.type)
    ) {
      throw new CustomError(
        'Error, must provide a valid OTC type (sell or buy).'
      );
    }
    if (
      req.body.user_to === undefined ||
      (await checkUserInSessionByName(session_id, req.body.user_to)) === null
    ) {
      throw new CustomError(
        'Error, must provide a valid username within the session.'
      );
    }
    const user_to_id = await checkUserInSessionByName(
      session_id,
      req.body.user_to
    );
    if (req.body.user_to === undefined || user_to_id === null) {
      if (
        req.body.volume_mwh === undefined ||
        req.body.volume_mwh === '' ||
        isNaN(Number(req.body.volume_mwh)) ||
        Number(req.body.volume_mwh <= 0)
      ) {
        throw new CustomError(
          'Error, please provide a positive numeric value for the bid volume_mwh'
        );
      }
    }
    if (
      req.body.price_eur_per_mwh === undefined ||
      req.body.price_eur_per_mwh === '' ||
      isNaN(Number(req.body.price_eur_per_mwh))
    ) {
      throw new CustomError(
        'Error, please provide a numeric value for the bid price_eur_per_mwh'
      );
    }

    /**
     * Insertion into DB
     */
    const otc: OTCEnergyExchange = {
      id: uuid(),
      user_from_id: user_id,
      user_to_id: user_to_id,
      session_id: session_id,
      phase_no: phase_infos.phase_no,
      type: req.body.type,
      volume_mwh: Number(req.body.volume_mwh),
      price_eur_per_mwh: Number(req.body.price_eur_per_mwh),
      status: 'pending',
    };

    await db.query(
      `INSERT INTO otc_exchanges
        (
          id,
          user_from_id,
          user_to_id,
          session_id,
          phase_no,
          type,
          volume_mwh,
          price_eur_per_mwh,
          status
        )
      VALUES
        ($1, $2, $3, $4, $5, $6, $7, $8, $9);`,
      [
        otc.id,
        otc.user_from_id,
        otc.user_to_id,
        otc.session_id,
        otc.phase_no,
        otc.type,
        otc.volume_mwh,
        otc.price_eur_per_mwh,
        otc.status,
      ]
    );

    /**
     * Send response and notify user_to of the new OTC
     */
    res.status(201).json({ otc_id: otc.id });
    notifyUser(session_id, user_to_id, 'new-otc', {
      id: otc.id,
      user_from: user.name,
      user_to: req.body.user_to,
      session_id: otc.session_id,
      phase_no: otc.phase_no,
      type: otc.type,
      volume_mwh: otc.volume_mwh,
      price_eur_per_mwh: otc.price_eur_per_mwh,
      status: otc.status,
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
 * Accept an OTC exchange. Can only be done by the OTC's user_to.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function acceptOTC(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    const session_id = req.params.session_id;
    const user_id = req.params.user_id;
    const otc_id = req.params.otc_id;

    /**
     * DB checks
     */
    const session = await getSession(session_id);
    if (session === null) {
      throw new CustomError('Error, no session found with this ID', 404);
    }
    const user = await getUser(session_id, user_id);
    if (user === null) {
      throw new CustomError('Error, no user found with this ID', 404);
    }
    const phase_infos = await getLastPhaseInfos(session_id);
    if (!phase_infos.plannings_allowed) {
      throw new CustomError('Error, cannot update an OTC exchange');
    }

    /**
     * OTC specific validation
     */
    const otc = await getOTCByID(otc_id);
    if (otc === null) {
      throw new CustomError('Error, no OTC found with this ID', 404);
    }
    if (otc.user_to_id !== user_id) {
      throw new CustomError('Error, not allowed to modify this OTC', 403);
    }
    if (otc.status === 'accepted' || otc.status === 'rejected') {
      throw new CustomError('Error, OTC has already been accepted/rejected');
    }

    /**
     * Update DB and notify user_from and user_to
     */
    await db.query(
      `UPDATE otc_exchanges
      SET status='accepted'
      WHERE id=$1;`,
      [otc_id]
    );
    res.end();
    const update = {
      otc_id: otc.id,
      status: 'accepted',
    };
    notifyUser(session_id, otc.user_from_id, 'otc-update', update);
    notifyUser(session_id, otc.user_to_id, 'otc-update', update);
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
 *Reject and OTC exchange. Can only be done by the OTC's user_to.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function rejectOTC(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    const session_id = req.params.session_id;
    const user_id = req.params.user_id;
    const otc_id = req.params.otc_id;

    /**
     * DB checks
     */
    const session = await getSession(session_id);
    if (session === null) {
      throw new CustomError('Error, no session found with this ID', 404);
    }
    const user = await getUser(session_id, user_id);
    if (user === null) {
      throw new CustomError('Error, no user found with this ID', 404);
    }
    const phase_infos = await getLastPhaseInfos(session_id);
    if (!phase_infos.plannings_allowed) {
      throw new CustomError('Error, cannot update an OTC exchange');
    }

    /**
     * OTC specific validation
     */
    const otc = await getOTCByID(otc_id);
    if (otc === null) {
      throw new CustomError('Error, no OTC found with this ID', 404);
    }
    if (otc.user_to_id !== user_id) {
      throw new CustomError('Error, not allowed to modify this OTC', 403);
    }
    if (otc.status === 'accepted' || otc.status === 'rejected') {
      throw new CustomError('Error, OTC has already been accepted/rejected');
    }

    /**
     * Update DB and notify user_from and user_to
     */
    await db.query(
      `UPDATE otc_exchanges
      SET status='rejected'
      WHERE id=$1;`,
      [otc_id]
    );
    res.end();
    const update = {
      otc_id: otc.id,
      status: 'rejected',
    };
    notifyUser(session_id, otc.user_from_id, 'otc-update', update);
    notifyUser(session_id, otc.user_to_id, 'otc-update', update);
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
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/otc`,
  getUserOTCsRoute
);
router.post(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/otc`,
  postOTC
);
router.put(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/otc/:otc_id(${uuid_regex})/accept`,
  acceptOTC
);
router.put(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/otc/:otc_id(${uuid_regex})/reject`,
  rejectOTC
);

export default router;
