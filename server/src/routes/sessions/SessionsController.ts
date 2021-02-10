import { Application } from "express";
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

  init(app: Application): void {}
}
