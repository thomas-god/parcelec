import Vue from "vue";
import Vuex, { StoreOptions } from "vuex";
import { user, UserState } from "./user";
import { session, SessionState } from "./session";
import { webSocket, WebSocketState } from "./webSocket";
import { portfolio, PortfolioState } from "./portfolio";
import { bids, BidsState } from "./bids";
import { results, ResultsState } from "./results";

Vue.use(Vuex);

export interface RootState {
  version: string;
  api_url: string;
  ws_url: string;
  user: UserState;
  session: SessionState;
  ws: WebSocketState;
  portfolio: PortfolioState;
  bids: BidsState;
  results: ResultsState;
}

const state: RootState = {
  version: "0.1.0",
  api_url: process.env.VUE_APP_API_URL,
  ws_url: process.env.VUE_APP_WS_URL,
} as RootState;

const store: StoreOptions<RootState> = {
  state: state,
  modules: {
    user,
    session,
    webSocket,
    portfolio,
    bids,
    results,
  },
};

export default new Vuex.Store<RootState>(store);
