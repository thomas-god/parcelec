import Vuex, {
  Module,
  GetterTree,
  MutationTree,
  ActionTree,
  Commit,
  ActionContext,
  Dispatch,
} from "vuex";
import { RootState } from "./index";

export interface ClientMessage {
  username: string;
  data: string;
  date: string;
}

export interface WebSocketState {
  ws: WebSocket | null;
  session_state: "NOT_REGISTERED" | "OPEN" | "RUNNING" | "CLOSE";
  messages: ClientMessage[];
}

// ------------------------ STATE -------------------------
export const state: WebSocketState = {
  ws: null,
  session_state: "NOT_REGISTERED",
  messages: [],
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<WebSocketState, RootState> = {
  /**
   * Open the WebSocket connection.
   */
  async openWebSocket(context): Promise<void> {
    // Close existing WebSocket connection and clear messages
    if (state.ws?.OPEN) state.ws.close();
    state.messages = [];

    // Open new WebSocket connection
    const ws_url = context.rootState.ws_url;
    const session_id = context.rootState.session.id;
    const user_id = context.rootState.user.user_id;
    const username = context.rootState.user.username;
    const socket = new WebSocket(
      `${ws_url}/auction?auction_id=${session_id}&user_id=${user_id}&username=${username}`
    );

    // Dirty hack to get the WS alive despite Nginx timeout
    const inter = setInterval(() => socket.send("{}"), 30000);
    socket.addEventListener("close", () => {
      onCloseCallback(context.commit);
      clearInterval(inter);
    });

    socket.addEventListener("message", (event) =>
      onMessageCallback(context.commit, context.dispatch, event)
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
          session_id: context.rootState.session.id,
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
  commit: Commit,
  dispatch: Dispatch,
  event: any
): void {
  try {
    const message = JSON.parse(event.data);
    console.log(message.reason);
    if (message.reason === "message") {
      commit("ADD_MESSAGE", message);
    } else if (message.username === "SERVER") {
      const opts = { root: true };
      switch (message.reason) {
        case "users-list-update":
          commit("session/SET_USERS", message.data, opts);
          break;
        case "reset-game-ready":
          dispatch("session/loadSessionTimings", {}, opts);
          commit("user/SET_GAME_READY", false, opts);
          break;
        case "new-game-phase":
          dispatch("session/loadSessionContent", {}, opts);
          dispatch("portfolio/loadPortfolioContent", {}, opts);
          commit("bids/SET_BIDS", [], opts);
          commit("bids/SET_CLEARING", [], opts);
          commit("bids/SET_ENERGY_EXCHANGES", [], opts);
          break;
        case "clearing-started":
          commit("session/SET_CAN_BID", false, opts);
          break;
        case "clearing-finished":
          commit("session/SET_CLEARING_AVAILABLE", true, opts);
          dispatch("bids/loadClearingContent", {}, opts);
          break;
        case "plannings-closed":
          commit("session/SET_CAN_POST_PLANNING", false, opts);
          dispatch("portfolio/resetPlanning", {}, opts);
          break;
        case "results-available":
          commit("session/SET_RESULTS_AVAILABLE", true, opts);
          dispatch("results/loadResultsContent", {}, opts);
          dispatch("bids/loadMarketBids", {}, opts);
          break;
      }
    }
  } catch (error) {
    console.log(error);
  }
}
