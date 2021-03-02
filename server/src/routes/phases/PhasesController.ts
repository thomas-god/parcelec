import { Application, Request, Response } from "express";
import { Dependencies } from "../../di.context";

export class PhasesController {
  private PhasesService: Dependencies["PhasesService"];

  constructor({
    PhasesService,
  }: {
    PhasesService: Dependencies["PhasesService"];
  }) {
    this.PhasesService = PhasesService;
  }

  init(app: Application): void {
    /**
     * @swagger
     * components:
     *  schemas:
     *    PhaseInfos:
     *      type: object
     *      properties:
     *        phase_no:
     *          schema: integer
     *          description: Phase number.
     *        status:
     *          schema: string
     *          enum: [open, closed]
     *          description: Phase status
     */

    /**
     * @swagger
     * /session/{sessionID}/phase:
     *  get:
     *    tags:
     *      - phase
     *    summary: Get the last phase of the current session
     *    operationId: getPhase
     *    parameters:
     *      - in: path
     *        name: sessionID
     *        schema:
     *          type: string
     *        required: true
     *        description: SessionID
     *    responses:
     *      '200':
     *        description: Phase information
     *        content:
     *          application/json:
     *            schema:
     *              type:
     *                $ref: '#/components/schemas/PhaseInfos'
     */
    app.get(
      "/session/:sessionID/phase",
      async (req: Request, res: Response) => {
        try {
          const sessionId = req.params.sessionID as string;

          const phase = await this.PhasesService.getPhaseInfos(sessionId);
          if (phase === undefined) {
            res.status(204).end();
          } else {
            res
              .status(200)
              .json({ phase_no: phase.phaseNo, status: phase.status });
          }
        } catch (err) {
          res.status(400).send({ message: err.message });
        }
      }
    );
  }
}
