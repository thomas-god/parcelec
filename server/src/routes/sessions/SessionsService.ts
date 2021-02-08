import { container, Dependencies } from '../../di.context'

export class SessionsService {
    private SessionsDAO : Dependencies["SessionsDAO"];

    constructor() {
        this.SessionsDAO = container.resolve("SessionsDAO");
    }
}