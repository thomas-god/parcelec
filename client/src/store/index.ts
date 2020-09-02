import Vue from "vue";
import Vuex, { StoreOptions } from "vuex";
import { user } from "./user";

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
  },
};

export default new Vuex.Store<RootState>(store);
