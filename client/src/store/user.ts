import Vuex, { Module, GetterTree, MutationTree, ActionTree } from "vuex";
import { RootState } from "./index";

export interface UserState {
  username: string;
  user_id: string;
  ready: boolean;
}

// ------------------------ STATE -------------------------
export const state: UserState = {
  username: "",
  user_id: "",
  ready: false,
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<UserState, RootState> = {
  setUsername({ commit }, username: string): void {
    commit("SET_USERNAME", username);
    commit(
      "session/PUSH_NEW_USER",
      { name: username, ready: false },
      { root: true }
    );
  },
  setUserID({ state, commit, dispatch }, user_id: string): void {
    commit("SET_USER_ID", user_id);
    dispatch("webSocket/openWebSocket", null, { root: true });
  },
  setReadyStatus({ commit }): void {
    commit("SET_READY");
  },
};

// ------------------------ MUTATIONS -------------------------
export const mutations: MutationTree<UserState> = {
  SET_USERNAME(state, payload: string): void {
    state.username = payload;
  },
  SET_USER_ID(state, payload: string): void {
    state.user_id = payload;
  },
  SET_READY(state): void {
    state.ready = true;
  },
};

// ------------------------ GETTERS -------------------------
export const getters: GetterTree<UserState, RootState> = {
  username(state): string {
    return state.username;
  },
  user_id(state): string {
    return state.user_id;
  },
  user_ready(state): boolean {
    return state.ready;
  },
};

// ------------------------ MODULE -------------------------
export const user: Module<UserState, RootState> = {
  namespaced: true,
  state,
  getters,
  actions,
  mutations,
};
