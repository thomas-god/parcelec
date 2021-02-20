import { Application, Request, Response } from "express";
import { Dependencies } from "../../di.context";

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
  }
}
