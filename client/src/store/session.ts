import Vue from "vue";
import Vuex, { Module, GetterTree, MutationTree, ActionTree } from "vuex";
import { RootState } from "./index";

export interface User {
  name: string;
  ready: boolean;
}

export interface Session {
  name: string;
  id: string;
  status: "open" | "running" | "closed";
  users: User[];
  can_bid: boolean;
  phase_infos?: {
    start_time: Date;
    clearing_time: Date;
    planning_time: Date;
  };
}

export interface SessionState extends Session {}

// ------------------------ STATE -------------------------
export const state: SessionState = {
  name: "",
  id: "",
  status: "open",
  users: [],
  can_bid: false,
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<SessionState, RootState> = {
  async setSession({ commit }, payload: Session): Promise<void> {
    const users = await getUsersList(payload.id);
    commit("SET_SESSION", { ...payload, users });
  },
  async updateUsersList({ state, commit }): Promise<void> {
    const users = await getUsersList(state.id);
    commit("SET_USERS", users);
  },
  setStatus({ state, commit }, status: Session["status"]): void {
    if (state.status === "open" && status === "running") {
      commit("UPDATE_CAN_BID", true);
    }
    commit("UPDATE_STATUS", status);
  },
  updateBidAbility({ commit }, bid_ability: boolean): void {
    commit("UPDATE_CAN_BID", bid_ability);
  },
};

// ------------------------ MUTATIONS -------------------------
export const mutations: MutationTree<SessionState> = {
  SET_SESSION(state, session: Session): void {
    state.id = session.id;
    state.name = session.name;
    state.status = session.status;
    state.users = session.users;
  },
  SET_USERS(state, users: User[]): void {
    state.users = users;
  },
  SET_PHASE_INFOS(state, phase_infos): void {
    Vue.set(state, "phase_infos", {
      start_time: new Date(phase_infos.start_time),
      clearing_time: new Date(phase_infos.clearing_time),
      planning_time: new Date(phase_infos.planning_time),
    });
  },
  PUSH_NEW_USER(state, new_user: User): void {
    state.users.push(new_user);
  },
  UPDATE_STATUS(state, status: Session["status"]): void {
    state.status = status;
  },
  UPDATE_CAN_BID(state, can_bid: boolean): void {
    state.can_bid = can_bid;
  },
};

// ------------------------ GETTERS -------------------------
export const getters: GetterTree<SessionState, RootState> = {
  session(state): Session {
    return state;
  },
  session_id(state): string {
    return state.id;
  },
  session_name(state): string {
    return state.name;
  },
  session_status(state): string {
    return state.status;
  },
  can_bid(state): boolean {
    return state.can_bid;
  },
  phase_infos(state): Session["phase_infos"] {
    return state.phase_infos;
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

// ------------------------ Helper functions ---------------
async function getUsersList(session_id: string): Promise<User[]> {
  const res = await fetch(
    `${process.env.VUE_APP_API_URL}/session/${session_id}`,
    {
      method: "GET",
    }
  );
  if (res.status === 200) {
    const body = await res.json();
    return body.users as User[];
  } else {
    return [];
  }
}
