import Vue from "vue";
import Vuex, { StoreOptions } from "vuex";
import { user, UserState } from "./user";
import { auction, AuctionState } from "./auction";
import { webSocket, WebSocketState } from "./webSocket";

Vue.use(Vuex);

export interface RootState {
  version: string;
  api_url: string;
  user: UserState;
  auction: AuctionState;
  ws: WebSocketState;
}

const state: RootState = {
  version: "0.1.0",
  api_url: process.env.VUE_APP_API_URL,
} as RootState;

const store: StoreOptions<RootState> = {
  state: state,
  modules: {
    user,
    auction,
    webSocket,
  },
};

export default new Vuex.Store<RootState>(store);
