import Vue from "vue";
import Vuex, { StoreOptions } from "vuex";
import { user, UserState } from "./user";
import { session, SessionState } from "./session";
import { webSocket, WebSocketState } from "./webSocket";
import { portfolio, PortfolioState } from "./portfolio";
import { bids, BidsState } from "./bids";

Vue.use(Vuex);

export interface RootState {
  version: string;
  api_url: string;
  user: UserState;
  session: SessionState;
  ws: WebSocketState;
  portfolio: PortfolioState;
  bids: BidsState;
}

const state: RootState = {
  version: "0.1.0",
  api_url: process.env.VUE_APP_API_URL,
} as RootState;

const store: StoreOptions<RootState> = {
  state: state,
  modules: {
    user,
    session,
    webSocket,
    portfolio,
    bids,
  },
};

export default new Vuex.Store<RootState>(store);
