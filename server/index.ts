import express from "express";
import ws from "ws";
import cors from "cors";
import morgan from "morgan";
import sessions from "./src/routes/sessions";
import users from "./src/routes/users";
import bids from "./src/routes/bids";
import otc from "./src/routes/otc";
import portfolio from "./src/routes/portfolio";
import { onConnectionCallback } from "./src/routes/websocket";
import db, { buildDefaultDB } from './src/db'

const app = express();
const port = Number(process.env.PORT) || 3000;

(async () => {
  await buildDefaultDB()
  console.log((await db.query(`
    SELECT *
    FROM films;
   `, [])).rows)
})();

app.use(cors());
app.use(express.json());
app.use(morgan("common"));
app.use("/", sessions);
app.use("/", users);
app.use("/", bids);
app.use("/", otc);
app.use("/", portfolio);

const wsServer = new ws.Server({ noServer: true, clientTracking: true });
wsServer.on("connection", onConnectionCallback);

const server = app.listen(port, (err) => {
  if (err) {
    return console.error(err);
  }
  return console.log(`server is listening on ${port}`);
});

server.on("upgrade", (request, socket, head) => {
  wsServer.handleUpgrade(request, socket, head, (socket) => {
    wsServer.emit("connection", socket, request);
  });
});
