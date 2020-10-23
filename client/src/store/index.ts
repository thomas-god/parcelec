import Vue from "vue";
import Vuex, { StoreOptions, MutationTree, GetterTree } from "vuex";
import { user, UserState } from "./user";
import { session, SessionState } from "./session";
import { webSocket, WebSocketState } from "./webSocket";
import { portfolio, PortfolioState } from "./portfolio";
import { bids, BidsState } from "./bids";
import { otcs, OTCState } from "./otc";
import { results, ResultsState } from "./results";

Vue.use(Vuex);

export interface Notifications {
  chat: boolean;
  market: boolean;
}

export interface RootState {
  version: string;
  api_url: string;
  ws_url: string;
  tuto_step: number;
  notifications: Notifications;
  user: UserState;
  session: SessionState;
  ws: WebSocketState;
  portfolio: PortfolioState;
  bids: BidsState;
  otcs: OTCState;
  results: ResultsState;
}

const state: RootState = {
  version: "0.1.0",
  api_url: process.env.VUE_APP_API_URL,
  ws_url: process.env.VUE_APP_WS_URL,
  tuto_step: 0,
  notifications: {
    chat: false,
    market: false,
  },
} as RootState;

const getters: GetterTree<RootState, RootState> = {
  notification_chat(state): boolean {
    return state.notifications.chat;
  },
  notification_market(state, _, rootState, rootGetters): boolean {
    return state.notifications.market || rootGetters["otcs/n_pending_otcs"] > 0;
  },
};

const mutations: MutationTree<RootState> = {
  SET_CHAT_NOTIFICATION(state, flag: boolean): void {
    state.notifications.chat = flag;
  },
  SET_MARKET_NOTIFICATION(state, flag: boolean): void {
    state.notifications.market = flag;
  },
};

const store: StoreOptions<RootState> = {
  state: state,
  mutations: mutations,
  getters: getters,
  modules: {
    user,
    session,
    webSocket,
    portfolio,
    bids,
    otcs,
    results,
  },
};

export default new Vuex.Store<RootState>(store);
