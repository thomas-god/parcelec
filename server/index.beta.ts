import { createServer } from './src/server'
import getContext from './src/di.context'

const container = getContext()

const app = createServer(container);
const port = 3000;

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