import express from "express";
import ws from "ws";
import cors from "cors";
import morgan from "morgan";
import user, { checkUsernameExists } from "./src/routes/user";

const app = express();
const port = 3000;
app.get("/", (req, res) => {
  res.send("The sedulous hyena ate the antelope!");
});

app.use(cors());
app.use(express.json());
app.use(morgan("common"));
app.use("/user", user);

interface ClientMessage {
  username: string;
  message: string;
  date: Date;
}
const messages: ClientMessage[] = [];

const wsServer = new ws.Server({ noServer: true, clientTracking: true });
wsServer.on("connection", (socket: ws) => {
  console.log("Web socket connection open");

  socket.on("message", (message: string) => {
    const msg = JSON.parse(message) as ClientMessage;
    msg.date = new Date();

    if (checkUsernameExists(msg.username)) {
      messages.push(msg);
      wsServer.clients.forEach((client) => {
        client.send(JSON.stringify(msg));
      });
    }
  });
});

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
