import { Application, Request, Response } from "express";
import { Dependencies } from "../../di.context";

export class UsersController {
  private UsersService: Dependencies["UsersService"];

  constructor({
    UsersService,
  }: {
    UsersService: Dependencies["UsersService"];
  }) {
    this.UsersService = UsersService;
  }

  init(app: Application) {
    /**
     * @swagger
     * /session/{sessionID}/user:
     *  put:
     *    tags:
     *      -user
     *    summary: Register a new user to a session.
     *    operationId: putUser
     *    parameters:
     *      - in: path
     *        name: sessionID
     *        schema:
     *          type: string
     *        required: true
     *        description: Session UUID.
     *      - in: query
     *        name: username
     *        schema:
     *          type: string
     *        required: true
     *        description: Username (must be unique within the session).
     *    responses:
     *      '201':
     *        description: User registered.
     *        content:
     *          application/json:
     *            schema:
     *              type: object
     *              properties:
     *                userId:
     *                  type: uuid
     */
    app.put("/session/:sessionID/user", async (req: Request, res: Response) => {
      try {
        const sessionId = req.params.sessionID as string;
        const username = req.query.username as string;

        const userId = await this.UsersService.registerUser(
          sessionId,
          username
        );

        res.status(201).json({ userId: userId });
      } catch (err) {
        res.status(400).send({ message: err.message });
      }
    });
  }
}
