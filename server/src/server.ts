import express from "express";
import { Application } from "express";
//import ws from "ws";
import cors from "cors";
import morgan from "morgan";
import { middleware as OpenAPIMiddleware } from "express-openapi-validator";
import dbmigrate from "db-migrate";

import { AwilixContainer } from "awilix";
import routes from "./routes/index";

export async function createServer(
  container: AwilixContainer
): Promise<Application> {
  const app = express();

  app.use(cors());
  app.use(express.json());
  app.use(morgan("common"));

  app.use(
    OpenAPIMiddleware({
      apiSpec: "./openapi.yaml",
      validateRequests: true,
      validateResponses: false,
    })
  );

  app.use((err, req, res, next) => {
    // format error
    res.status(err.status || 500).json({
      message: err.message,
    });
  });

  const dbm = dbmigrate.getInstance(true, {
    env: "test",
    config: "database.json",
  });
  await dbm.up();

  routes(container, app);

  return app;
}

// // const wsServer = new ws.Server({ noServer: true, clientTracking: true });
// // wsServer.on("connection", onConnectionCallback);

// const server = app.listen(port, (err) => {
//   if (err) {
//     return console.error(err);
//   }
//   return console.log(`server is listening on ${port}`);
// });

// // server.on("upgrade", (request, socket, head) => {
// //   wsServer.handleUpgrade(request, socket, head, (socket) => {
// //     wsServer.emit("connection", socket, request);
// //   });
// // });

// export default server;
