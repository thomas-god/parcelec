import { Application, Request, Response } from 'express';
import { Dependencies } from '../../di.context';

export class UsersController {
  private UsersService: Dependencies['UsersService'];

  constructor({
    UsersService,
  }: {
    UsersService: Dependencies['UsersService'];
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
     *    operationId: registerUser
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
    app.put('/session/:sessionID/user', async (req: Request, res: Response) => {
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

    /**
     * @swagger
     * /session/{sessionID}/user/{userID}/ready:
     *  put:
     *    tags:
     *      -user
     *    summary: Mark a user ready.
     *    operationId: markUserReady
     *    parameters:
     *      - in: path
     *        name: sessionID
     *        schema:
     *          type: string
     *        required: true
     *        description: Session ID.
     *      - in: path
     *        name: userID
     *        schema:
     *          type: string
     *        required: true
     *        description: User ID.
     *    responses:
     *      '200':
     *        description: User marked ready.
     */
    app.put(
      '/session/:sessionID/user/:userID/ready',
      async (req: Request, res: Response) => {
        try {
          const sessionId = req.params.sessionID as string;
          const userId = req.params.userID as string;

          await this.UsersService.markUserReady(sessionId, userId);
          res.status(200).end();
        } catch (err) {
          res.status(400).send({ message: err.message });
        }
      }
    );
  }
}
