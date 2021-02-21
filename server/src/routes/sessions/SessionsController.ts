import { Application, Request, Response } from "express";
import { Dependencies } from "../../di.context";
import { SessionStatus } from "./types";

export class SessionsController {
  private SessionsService: Dependencies["SessionsService"];
  private UsersService: Dependencies["UsersService"];

  constructor({
    SessionsService,
    UsersService,
  }: {
    SessionsService: Dependencies["SessionsService"];
    UsersService: Dependencies["UsersService"];
  }) {
    this.SessionsService = SessionsService;
    this.UsersService = UsersService;
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
     *              type: array
     *              items:
     *                type:
     *                  $ref: '#/components/schemas/SessionItem'
     */
    app.get("/sessions", async (req: Request, res: Response) => {
      try {
        const sessions = await this.SessionsService.getSessionList(
          SessionStatus.open
        );
        res.status(200).json(sessions);
      } catch (err) {
        res.status(400).send(err.message);
      }
    });

    /**
     * @swagger
     * components:
     *  schemas:
     *    UserItem:
     *      type: object
     *      properties:
     *        name:
     *          schema: string
     *          description: Username.
     *        status:
     *          schema: boolean
     *          description: User ready or not.
     */

    /**
     * @swagger
     * /session/{sessionID}/users:
     *  get:
     *    tags:
     *      - session
     *    summary: Get a list of session's users.
     *    operationId: getSessionUsers
     *    parameters:
     *      - in: path
     *        name: sessionID
     *        schema:
     *          type: string
     *        required: true
     *        description: Session ID.
     *    responses:
     *      '200':
     *        description: List of users.
     *        content:
     *          application/json:
     *            schema:
     *              type: array
     *              items:
     *                type:
     *                  $ref: '#/components/schemas/UserItem'
     */
    app.get(
      "/session/:sessionID/users",
      async (req: Request, res: Response) => {
        try {
          const sessionId = req.params.sessionID as string;

          const users = await this.UsersService.getSessionUsers(sessionId);
          res.status(200).json(users);
        } catch (err) {
          res.status(400).send({ message: err.message });
        }
      }
    );
  }
}
