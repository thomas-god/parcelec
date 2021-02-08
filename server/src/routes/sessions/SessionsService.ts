import { Dependencies } from '../../di.context'

export class SessionsService {
    private SessionsDAO : Dependencies["SessionsDAO"];

    constructor({ SessionsDAO }: { SessionsDAO: Dependencies["SessionsDAO"]}) {
        this.SessionsDAO = SessionsDAO
    }
}