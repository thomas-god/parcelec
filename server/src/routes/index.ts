import { AwilixContainer } from "awilix";
import { Application } from "express";
import { Dependencies } from "../di.context";

export default function (container: AwilixContainer, app: Application) {
  (container.resolve("BidsController") as Dependencies["BidsController"]).init(
    app
  );

  (container.resolve(
    "SessionsController"
  ) as Dependencies["SessionsController"]).init(app);

  (container.resolve(
    "UsersController"
  ) as Dependencies["UsersController"]).init(app);

  (container.resolve(
    "PhasesController"
  ) as Dependencies["PhasesController"]).init(app);
}
