import { Application, Request, Response } from 'express';
import { Dependencies } from '../../di.context';
import { BidTypes } from './types';

export class BidsController {
  private BidsService: Dependencies['BidsService'];

  constructor({ BidsService }: { BidsService: Dependencies['BidsService'] }) {
    this.BidsService = BidsService;
  }

  init(app: Application): void {
    /**
     * @swagger
     * components:
     *  schemas:
     *    NewBidBody:
     *      type: object
     *      required:
     *      - type
     *      - volume
     *      - price
     *      properties:
     *        type:
     *          type: string
     *          enum: [sell, buy]
     *        volume:
     *          type: integer
     *        price:
     *          type: integer
     */

    /**
     * @swagger
     * components:
     *  schemas:
     *    BidResponse:
     *      type: object
     *      required:
     *      - id
     *      properties:
     *        id:
     *          type: string
     */

    /**
     * @swagger
     * /session/{sessionID}/user/{userID}/bid:
     *  post:
     *    summary: post a new bid
     *    parameters:
     *      - in: path
     *        name: sessionID
     *        schema:
     *          type: string
     *        required: true
     *        description: Session UUID
     *      - in: path
     *        name: userID
     *        schema:
     *          type: string
     *        required: true
     *        description: User UUID
     *    requestBody:
     *      description: Bid to post
     *      required: true
     *      content:
     *        application/json:
     *          schema:
     *            type: object
     *            required:
     *             - bid
     *            properties:
     *              bid:
     *                $ref: '#/components/schemas/NewBidBody'
     *    responses:
     *      '201':
     *        description: bid inserted
     *        content:
     *          application/json:
     *            schema:
     *              $ref: '#/components/schemas/BidResponse'
     */
    app.post(
      `/session/:sessionID/user/:userID/bid`,
      async (req: Request, res: Response) => {
        try {
          const sessionId = req.params.sessionID;
          const userId = req.params.userID;
          const type: BidTypes = req.body.bid.type as BidTypes;
          const price = Number(req.body.bid.price);
          const volume = Number(req.body.bid.volume);

          const bidId = await this.BidsService.postUserBid(
            sessionId,
            userId,
            type,
            volume,
            price
          );
          res.status(201).json({ id: bidId });
        } catch (err) {
          res.status(400).send({ message: err.message });
        }
      }
    );
  }
}
