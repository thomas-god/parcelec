import { Application, Request, Response } from "express";
import { Dependencies } from "../../di.context";
import { SessionStatus } from "./types";

export class SessionsController {
  private SessionsService: Dependencies["SessionsService"];

  constructor({
    SessionsService,
  }: {
    SessionsService: Dependencies["SessionsService"];
  }) {
    this.SessionsService = SessionsService;
  }

  init(app: Application): void {
    /**
     * @swagger
     * /session:
     *  put:
     *    tags:
     *      - session
     *    summary: Create a new game session
     *    operationId: putSession
     *    parameters:
     *      - in: query
     *        name: sessionName
     *        schema:
     *          type: string
     *        required: true
     *        description: New session name
     *    responses:
     *      '201':
     *        description: session created
     *        content:
     *          application/json:
     *            schema:
     *              type: object
     *              properties:
     *                sessionId:
     *                  type: uuid
     */
    app.put("/session", async (req: Request, res: Response) => {
      try {
        const sessionName: string = req.query.sessionName as string;
        const sessionId = await this.SessionsService.createSession(sessionName);
        res.status(201).json({ sessionId: sessionId });
      } catch (err) {
        res.status(500).send(err.message);
      }
    });

    /**
     * @swagger
     * components:
     *  schemas:
     *    SessionItem:
     *      type: object
     *      properties:
     *        id:
     *          type: string
     *          description: Session ID.
     *        name:
     *          type: string
     *          description: Session name.
     */

    /**
     * @swagger
     * /sessions:
     *  get:
     *    tags:
     *      - session
     *    summary: Get a list of current open games.
     *    operationId: getOpenSessionsList
     *    responses:
     *      '200':
     *        description: List of open sessions.
     *        content:
     *          application/json:
     *            schema:
     *              $ref: '#/components/schemas/SessionItem'
     */
    app.get("/sessions", async (req: Request, res: Response) => {
      try {
        const sessions = await this.SessionsService.getSessionList(
          SessionStatus.open
        );
        res.status(200).json({ sessions: sessions });
      } catch (err) {
        res.status(400).send(err.message);
      }
    });
  }
}
