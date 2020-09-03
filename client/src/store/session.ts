import Vuex, { Module, GetterTree, MutationTree, ActionTree } from "vuex";
import { RootState } from "./index";

export interface Session {
  name: string;
  id: string;
  status: "Open" | "Running" | "Close";
}

export interface SessionState {
  session: Session;
}

// ------------------------ STATE -------------------------
export const state: SessionState = {
  session: {
    name: "",
    id: "",
    status: "Close",
  },
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<SessionState, RootState> = {
  setSession({ commit }, payload: Session): void {
    commit("SET_SESSION", payload);
  },
};

// ------------------------ MUTATIONS -------------------------
export const mutations: MutationTree<SessionState> = {
  SET_SESSION(state, payload: Session): void {
    state.session = payload;
  },
};

// ------------------------ GETTERS -------------------------
export const getters: GetterTree<SessionState, RootState> = {
  session(state): Session {
    return state.session;
  },
  session_id(state): string {
    return state.session.id;
  },
};

// ------------------------ MODULE -------------------------
export const session: Module<SessionState, RootState> = {
  namespaced: true,
  state,
  getters,
  actions,
  mutations,
};
