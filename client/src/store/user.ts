import Vuex, { Module, GetterTree, MutationTree, ActionTree } from "vuex";
import { RootState } from "./index";

export interface UserState {
  username: string;
}

// ------------------------ STATE -------------------------
export const state: UserState = {
  username: "",
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<UserState, RootState> = {
  setUsername({ commit }, payload: string): void {
    commit("SET_USERNAME", payload);
  },
};

// ------------------------ MUTATIONS -------------------------
export const mutations: MutationTree<UserState> = {
  SET_USERNAME(state, payload: string): void {
    state.username = payload;
  },
};

// ------------------------ GETTERS -------------------------
export const getters: GetterTree<UserState, RootState> = {
  username(state): string {
    return state.username;
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
