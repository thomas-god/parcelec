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
  setUserID({ commit }, user_id: string): void {
    commit("SET_USER_ID", user_id);
  },
  async loadUserContent({ commit, rootState }): Promise<void> {
    const api_url = rootState.api_url;
    const session_id = rootState.session.id;
    const user_id = rootState.user.user_id;
    const user = await (
      await fetch(`${api_url}/session/${session_id}/user/${user_id}`, {
        method: "GET",
      })
    ).json();
    commit("SET_USERNAME", user.name);
    commit("SET_GAME_READY", user.ready);
  },
};

// ------------------------ MUTATIONS -------------------------
export const mutations: MutationTree<UserState> = {
  SET_USERNAME(state, username: string): void {
    state.username = username;
  },
  SET_USER_ID(state, user_id: string): void {
    state.user_id = user_id;
  },
  SET_GAME_READY(state, game_ready: boolean): void {
    state.ready = game_ready;
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
