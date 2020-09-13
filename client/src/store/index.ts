import Vue from "vue";
import Vuex, { StoreOptions } from "vuex";
import { user, UserState } from "./user";
import { auction, AuctionState } from "./auction";

Vue.use(Vuex);

export interface RootState {
  version: string;
  user: UserState;
  auction: AuctionState;
}

const state: RootState = {
  version: "0.1.0",
} as RootState;

const store: StoreOptions<RootState> = {
  state: state,
  modules: {
    user,
    auction,
  },
};

export default new Vuex.Store<RootState>(store);
