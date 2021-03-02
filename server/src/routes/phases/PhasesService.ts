import { Dependencies } from '../../di.context';
import { Phase } from './types';

export class PhasesService {
  private PhasesDAO: Dependencies['PhasesDAO'];
  private SessionsDAO: Dependencies['SessionsDAO'];
  private SessionsService: Dependencies['SessionsService'];

  constructor({
    PhasesDAO,
    SessionsDAO,
    SessionsService,
  }: {
    PhasesDAO: Dependencies['PhasesDAO'];
    SessionsDAO: Dependencies['SessionsDAO'];
    SessionsService: Dependencies['SessionsService'];
  }) {
    this.PhasesDAO = PhasesDAO;
    this.SessionsDAO = SessionsDAO;
    this.SessionsService = SessionsService;
  }

  async getPhaseInfos(sessionID: string): Promise<Phase> {
    await this.SessionsService.getSession(sessionID);

    return await this.PhasesDAO.getLastPhaseInfos(sessionID);
  }
}
