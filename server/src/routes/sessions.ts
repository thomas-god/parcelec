import { v4 as uuidv4 } from "uuid";
import ws from "ws";
import express from "express";
const router = express.Router();

export interface Session {
  name: string;
  id: string;
  status: "Open" | "Running" | "Close";
  users: User[];
}

export interface User {
  username: string;
  websocket?: ws;
  user_id: string;
}

export interface ClientMessage {
  username: string;
  date: Date;
  reason: "message" | "handshake";
  credentials: {
    session_id: string;
    user_id: string;
  };
  data: any;
}

const sessions: Session[] = [];

// ---------------------- Routing Functions

/**
 * Get the list of sessions that are currently open.
 */
router.get("/list_open", (req, res) => {
  res.send(
    JSON.stringify(
      sessions
        .filter((s) => s.status === "Open")
        .map((s) => {
          return { name: s.name, id: s.id };
        })
    )
  );
});

/**
 * Open a new session with a user provided name. Name must be unique.
 */
router.put("/open", (req, res) => {
  const session_name: string = req.body.session_name;
  if (session_name && !checkSessionNameExists(session_name)) {
    const session: Session = {
      name: session_name,
      id: uuidv4(),
      status: "Open",
      users: [],
    };
    sessions.push(session);
    res.send(session);
  } else {
    res.status(400).end("Error, unable to create a new session");
  }
});

/**
 * Start a session (i.e. put its status to 'Running') provided its ID.
 */
router.put("/start", (req, res) => {
  if (!req.body.session) {
    res.status(400).end("Error, no session object provided");
    return;
  }
  if (!req.body.session.id) {
    res.status(400).end("Error, no session ID provided");
    return;
  }
  if (!checkSessionIDExists(req.body.session.id)) {
    res
      .status(400)
      .end(
        "Error, the session ID provided does not match an existing sessions"
      );
    return;
  }
  const session = findSessionByID(req.body.session.id);
  if (session.status === "Running") {
    res.status(400).end("Error, the session is already running");
    return;
  }
  if (session.status === "Close") {
    res.status(400).end("Error, the session is closed");
    return;
  }
  session.status = "Running";
});

/**
 * Add a user by its username to the list of user of an open session.
 * Username must be unique.
 */
router.put("/register_user", (req, res) => {
  if (!req.body.session) {
    res.status(400).end("Error, no session object provided");
    return;
  }
  if (!req.body.session.id) {
    res.status(400).end("Error, no session ID provided");
    return;
  }
  if (!checkSessionIDExists(req.body.session.id)) {
    res
      .status(400)
      .end(
        "Error, the session ID provided does not match an existing sessions"
      );
    return;
  }
  if (!req.body.username) {
    res.status(400).end("Error, no username provided");
    return;
  }
  const session = findSessionByID(req.body.session.id);
  if (session.status !== "Open") {
    res.status(400).end("Error, the session is not open for registration");
    return;
  }
  if (session.users.some((u) => u.username === req.body.username)) {
    res.status(400).end("Error, the username already exist");
    return;
  }
  const user_id = uuidv4();
  session.users.push({
    username: req.body.username,
    user_id: user_id,
  });
  res.json({ user_id: user_id });
});

// ---------------------- WebSocket onConnection callback
export function onConnectionCallback(socket: ws): void {
  socket.on("message", (message: string) => {
    const msg = JSON.parse(message) as ClientMessage;
    msg.date = new Date();

    if (msg.reason === "message") {
      messageCallback(socket, msg);
    } else if (msg.reason === "handshake") {
      handshakeCallback(socket, msg);
    }
  });
}

/**
 * Handle the authentification of the WebSocket connection
 * @param socket Current socket
 * @param msg Incoming message
 */
function isAllowed(socket: ws, msg: ClientMessage): boolean {
  const session = findSessionByID(msg.credentials.session_id);
  if (!session) {
    socket.close(
      4000,
      "Error, the ID provided does not correspond to an existing session"
    );
    return false;
  }
  if (session.users.some((u) => u.user_id === msg.credentials.user_id)) {
    return true;
  } else {
    socket.close(
      4000,
      "Error, the ID provided does not correspond to an existing user in the session"
    );
    return false;
  }
}

/**
 * Handle the handshake between the client and the server
 * @param socket Current WebSocket
 * @param msg Incoming message
 */
function handshakeCallback(socket: ws, msg: ClientMessage): void {
  const session = sessions.find((s) => s.id === msg.credentials.session_id);
  if (session) {
    const user = session.users.find(
      (u) => u.user_id === msg.credentials.user_id
    );
    if (user) {
      console.log("doing handshake for ", msg.username);
      user.websocket = socket;
    }
  }
}

/**
 * Handle new message and spread it to the connected users
 * @param socket Current WebSocket
 * @param msg Incoming message
 */
function messageCallback(socket: ws, msg: ClientMessage): void {
  if (isAllowed(socket, msg)) {
    const session = sessions.find((s) => s.id === msg.credentials.session_id);
    session.users.forEach((u) => {
      u.websocket?.send(
        JSON.stringify({
          username: msg.username,
          date: msg.date,
          reason: "message",
          data: msg.data,
        })
      );
    });
  }
}

function sendConnectedUsersList(socket: ws, msg: ClientMessage): void {
  if (isAllowed(socket, msg)) {
    const session = findSessionByID(msg.credentials.session_id);
    socket.send(
      JSON.stringify({
        reason: "users_connected_list",
        data: session.users.map((u) => u.username),
      })
    );
  }
}

// ---------------------- Helper Functions

function checkSessionNameExists(session_name: string): boolean {
  return sessions.filter((session) => session.name === session_name).length > 0;
}

function checkSessionIDExists(session_id: string): boolean {
  return sessions.filter((s) => s.id === session_id).length === 1;
}

function findSessionByID(session_id: string): Session {
  if (checkSessionIDExists(session_id)) {
    return sessions.filter((s) => s.id === session_id)[0];
  }
}

export default router;
