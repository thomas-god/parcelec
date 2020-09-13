import Vuex, { Module, GetterTree, MutationTree, ActionTree } from "vuex";
import { RootState } from "./index";

export interface UserState {
  username: string;
  user_id: string;
}

// ------------------------ STATE -------------------------
export const state: UserState = {
  username: "",
  user_id: "",
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<UserState, RootState> = {
  setUsername({ commit }, username: string): void {
    commit("SET_USERNAME", username);
    commit("auction/PUSH_NEW_USER", username, { root: true });
  },
  setUserID({ state, commit, dispatch }, user_id: string): void {
    commit("SET_USER_ID", user_id);
    dispatch("webSocket/openWebSocket", null, { root: true });
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
};

// ------------------------ GETTERS -------------------------
export const getters: GetterTree<UserState, RootState> = {
  username(state): string {
    return state.username;
  },
  user_id(state): string {
    return state.user_id;
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
