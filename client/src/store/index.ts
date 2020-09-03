import Vue from "vue";
import Vuex, { StoreOptions } from "vuex";
import { user } from "./user";
import { session } from "./session";

Vue.use(Vuex);

export interface RootState {
  version: string;
}

const store: StoreOptions<RootState> = {
  state: {
    version: "0.1.0",
  },
  modules: {
    user,
    session,
  },
};

export default new Vuex.Store<RootState>(store);
