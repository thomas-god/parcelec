import express from "express";
//import ws from "ws";
import cors from "cors";
import morgan from "morgan";
import { middleware as OpenAPIMiddleware} from 'express-openapi-validator';

import routes from './src/routes/index'
import context from './src/di.context'

const app = express();
const port = 3000;

context()

app.use(cors());
app.use(express.json());
app.use(morgan("common"));

app.use(
  OpenAPIMiddleware({
    apiSpec: './openapi.yaml',
    validateRequests: true, // (default)
    validateResponses: true, // false by default
  }),
);

app.use((err, req, res, next) => {
  // format error
  res.status(err.status || 500).json({
    message: err.message,
    errors: err.errors,
  });
});

routes(app)


// const wsServer = new ws.Server({ noServer: true, clientTracking: true });
// wsServer.on("connection", onConnectionCallback);

const server = app.listen(port, (err) => {
  if (err) {
    return console.error(err);
  }
  return console.log(`server is listening on ${port}`);
});

// server.on("upgrade", (request, socket, head) => {
//   wsServer.handleUpgrade(request, socket, head, (socket) => {
//     wsServer.emit("connection", socket, request);
//   });
// });

export default server;