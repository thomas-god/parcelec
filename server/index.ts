import express from "express";
import ws from "ws";
import cors from "cors";
import morgan from "morgan";
import user from "./src/routes/user";
import auction from "./src/routes/auctions";
import { onConnectionCallback } from "./src/routes/websocket";

const app = express();
const port = 3000;

app.use(cors());
app.use(express.json());
app.use(morgan("common"));
app.use("/user", user);
app.use("/auction", auction);

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
