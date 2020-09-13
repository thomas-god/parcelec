import Vuex, {
  Module,
  GetterTree,
  MutationTree,
  ActionTree,
  Commit,
  ActionContext,
} from "vuex";
import { RootState } from "./index";

export interface ClientMessage {
  username: string;
  data: string;
  date: string;
}

export interface WebSocketState {
  ws: WebSocket | null;
  auction_state: "NOT_REGISTERED" | "OPEN" | "RUNNING" | "CLOSE";
  messages: ClientMessage[];
}

// ------------------------ STATE -------------------------
export const state: WebSocketState = {
  ws: null,
  auction_state: "NOT_REGISTERED",
  messages: [],
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<WebSocketState, RootState> = {
  async openWebSocket(context): Promise<void> {
    // Close existing WebSocket connection
    if (state.ws?.OPEN) state.ws.close();

    // Open new WebSocket connection
    const session_id = context.rootState.auction.id;
    const user_id = context.rootState.user.user_id;
    const username = context.rootState.user.username;
    const socket = new WebSocket(
      `ws://localhost:3000/auction?auction_id=${session_id}&user_id=${user_id}&username=${username}`
    );

    socket.addEventListener("close", () => onCloseCallback(context.commit));
    socket.addEventListener("message", (event) =>
      onMessageCallback(context, event)
    );

    context.commit("SET_WEBSOCKET", socket);
  },
  sendMsg(context, payload: string): void {
    console.log(payload);
    if (payload) {
      const msg = JSON.stringify({
        username: context.rootState.user.username,
        reason: "message",
        credentials: {
          auction_id: context.rootState.auction.id,
          user_id: context.rootState.user.user_id,
        },
        data: payload,
      });
      context.state.ws?.send(msg);
    }
  },
};

// ------------------------ MUTATIONS -------------------------
export const mutations: MutationTree<WebSocketState> = {
  SET_WEBSOCKET(state, payload: WebSocket): void {
    state.ws = payload;
  },
  ADD_MESSAGE(state, msg: ClientMessage): void {
    state.messages.push(msg);
  },
};

// ------------------------ GETTERS -------------------------
export const getters: GetterTree<WebSocketState, RootState> = {
  state(state): number {
    return state.ws ? state.ws.readyState : -1;
  },
};

// ------------------------ MODULE -------------------------
export const webSocket: Module<WebSocketState, RootState> = {
  namespaced: true,
  state,
  getters,
  actions,
  mutations,
};

// ------------------------ Utils functions -----------------
function onCloseCallback(commit: Commit): void {
  commit("SET_WEBSOCKET", null);
}

function onMessageCallback(
  context: ActionContext<WebSocketState, RootState>,
  event: any
): void {
  try {
    const message = JSON.parse(event.data);
    if (message.reason === "message") {
      context.commit("ADD_MESSAGE", message);
    } else if (message.reason === "new_user" && message.username === "SERVER") {
      if (
        context.rootState.auction.users.length !==
        message.data.nb_users - 1
      ) {
        // Need to refresh the whole users list as some seem to be missing
        context.dispatch("auction/updateUsersList", null, { root: true });
      } else {
        context.commit("auction/PUSH_NEW_USER", message.data.username, {
          root: true,
        });
      }
    }
  } catch (error) {
    console.log(error);
  }
}
