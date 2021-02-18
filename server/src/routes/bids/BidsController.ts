import { Application, Request, Response } from "express";
import { Dependencies } from "../../di.context";
import { uuid_regex } from "../utils";
import { BidInput } from "./BidsService";

export class BidsController {
  private BidsService: Dependencies["BidsService"];

  constructor({ BidsService }: { BidsService: Dependencies["BidsService"] }) {
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
     *      - volume_mwh
     *      - price_eur_per_mwh
     *      properties:
     *        type:
     *          type: string
     *          enum: [sell, buy]
     *        volume_mwh:
     *          type: integer
     *        price_eur_per_mwh:
     *          type: integer
     */

    /**
     * @swagger
     * components:
     *  schemas:
     *    BidId:
     *      type: object
     *      required:
     *      - id
     *      properties:
     *        id:
     *          type: string
     */

    /**
     * @swagger
     * /beta/session/{session_id}/user/{user_id}/bid:
     *  post:
     *    summary: post a new bid
     *    parameters:
     *      - in: path
     *        name: session_id
     *        schema:
     *          type: string
     *        required: true
     *        description: Session UUID
     *      - in: path
     *        name: user_id
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
     *              $ref: '#/components/schemas/BidId'
     */
    app.post(
      `/beta/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/bid`,
      async (req: Request, res: Response) => {
        const sessionId = req.params.session_id;
        const userId = req.params.user_id;
        const bidBody = req.body.bid as BidInput;

        try {
          const bid = await this.BidsService.postUserBid(
            sessionId,
            userId,
            bidBody
          );
          console.log(bid)
          res.status(201).json({ id: bid.id });
        } catch (error) {
          console.log(error.message)
          res.status(500).send(error.message)
        }
      }
    );
  }
}
