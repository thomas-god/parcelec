import { createContainer, AwilixContainer, asValue, asClass, InjectionMode } from 'awilix';

import { Database } from './db'

import { BidsDAO } from './routes/bids/BidsDAO'
import { BidsService } from './routes/bids/BidsService'
import { BidsController } from './routes/bids/BidsController'

// import { SessionsDAO } from './routes/sessions/SessionsDAO'
// import { SessionsService } from './routes/sessions/SessionsService'
// import { SessionsController } from './routes/sessions/SessionsController'

export const container: AwilixContainer = createContainer({
  injectionMode: InjectionMode.PROXY
});

export default function context(): void {
  container.register({
    db: asValue(new Database()),
    BidsDAO: asClass(BidsDAO),
    BidsService: asClass(BidsService),
    BidsController: asClass(BidsController),
    // SessionsDAO: asValue(new SessionsDAO()),
    // SessionsService: asValue(new SessionsService()),
    // SessionsController: asValue(new SessionsController()),
  })
}

export interface Dependencies {
  db: Database;
  BidsDAO: BidsDAO;
  BidsService: BidsService;
  BidsController: BidsController;
  // SessionsDAO: SessionsDAO;
  // SessionsService: SessionsService;
  // SessionsController: SessionsController;
}

