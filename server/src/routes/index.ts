import { AwilixContainer } from "awilix";
import { Application } from "express";
import { Dependencies } from "../di.context";

export default function (container: AwilixContainer, app: Application) {
  (container.resolve("BidsController") as Dependencies["BidsController"]).init(
    app
  );
}
