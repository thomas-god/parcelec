import {
  createContainer,
  AwilixContainer,
  asValue,
  asClass,
  InjectionMode,
} from "awilix";

import { Database } from "./db";

import { BidsDAO } from "./routes/bids/BidsDAO";
import { BidsService } from "./routes/bids/BidsService";
import { BidsController } from "./routes/bids/BidsController";

import { SessionsDAO } from "./routes/sessions/SessionsDAO";
import { SessionsService } from "./routes/sessions/SessionsService";
import { SessionsController } from "./routes/sessions/SessionsController";

import { UsersController } from "./routes/users/UsersController";
import { UsersService } from "./routes/users/UsersService";
import { UsersDAO } from "./routes/users/UsersDAO";

export default function getContext(): AwilixContainer {
  const container: AwilixContainer = createContainer({
    injectionMode: InjectionMode.PROXY,
  });

  container.register({
    db: asValue(new Database()),
    BidsDAO: asClass(BidsDAO),
    BidsService: asClass(BidsService),
    BidsController: asClass(BidsController),
    SessionsDAO: asClass(SessionsDAO),
    SessionsService: asClass(SessionsService),
    SessionsController: asClass(SessionsController),
    UsersDAO: asClass(UsersDAO),
    UsersService: asClass(UsersService),
    UsersController: asClass(UsersController),
  });

  return container;
}

export interface Dependencies {
  db: Database;
  BidsDAO: BidsDAO;
  BidsService: BidsService;
  BidsController: BidsController;
  SessionsDAO: SessionsDAO;
  SessionsService: SessionsService;
  SessionsController: SessionsController;
  UsersDAO: UsersDAO;
  UsersService: UsersService;
  UsersController: UsersController;
}
