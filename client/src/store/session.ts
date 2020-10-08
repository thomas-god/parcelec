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
  can_post_planning: boolean;
  clearing_available: boolean;
  results_available: boolean;
  phase_infos?: {
    phase_no: number;
    nb_phases: number;
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
  can_post_planning: false,
  clearing_available: false,
  results_available: false,
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<SessionState, RootState> = {
  setSessionID({ commit }, session_id: string): void {
    commit("SET_SESSION_ID", session_id);
  },
  /**
   * Master entry-point to load all game related information,
   * user, session, portfolio, etc. and open the WebSocket
   * connection. User ID and session ID must be set in their
   * respective stores before proceeding.
   */
  async loadGameContent(context): Promise<void> {
    // Cannot load if we don't have session ID and user ID
    if (context.rootState.user.user_id && context.rootState.session.id) {
      context.dispatch("user/loadUserContent", {}, { root: true });
      context.dispatch("session/loadSessionContent", {}, { root: true });
      context.dispatch("portfolio/loadPortfolioContent", {}, { root: true });
      context.dispatch("bids/loadBidsContent", {}, { root: true });
      context.dispatch("bids/loadClearingContent", {}, { root: true });
      context.dispatch("bids/loadMarketBids", {}, { root: true });
      context.dispatch("results/loadResultsContent", {}, { root: true });
      context.dispatch("webSocket/openWebSocket", {}, { root: true });
    }
  },
  /**
   * Load all session related information from the server into
   * the session store.
   */
  async loadSessionContent({ commit, rootState }) {
    const api_url = rootState.api_url;
    const session_id = rootState.session.id;
    const session = await (
      await fetch(`${api_url}/session/${session_id}`, {
        method: "GET",
      })
    ).json();
    commit("SET_NAME", session.name);
    commit("SET_STATUS", session.status);
    commit("SET_CAN_BID", session.bids_allowed);
    commit("SET_CAN_POST_PLANNING", session.plannings_allowed);
    commit("SET_CLEARING_AVAILABLE", session.clearing_available);
    commit("SET_RESULTS_AVAILABLE", session.results_available);
    commit("SET_USERS", session.users);
    if (session.phase_infos) {
      commit("SET_PHASE_INFOS", session.phase_infos);
    }
  },
  /**
   * Load session information related to timings.
   */
  async loadSessionTimings({ commit, rootState }) {
    const api_url = rootState.api_url;
    const session_id = rootState.session.id;
    const session = await (
      await fetch(`${api_url}/session/${session_id}`, {
        method: "GET",
      })
    ).json();
    if (session.phase_infos) {
      commit("SET_PHASE_INFOS", session.phase_infos);
    }
  },
};

// ------------------------ MUTATIONS -------------------------
export const mutations: MutationTree<SessionState> = {
  SET_NAME(state, name: string): void {
    state.name = name;
  },
  SET_SESSION_ID(state, session_id: string): void {
    state.id = session_id;
  },
  SET_STATUS(state, status: Session["status"]): void {
    state.status = status;
  },
  SET_USERS(state, users: User[]): void {
    state.users = users;
  },
  PUSH_NEW_USER(state, new_user: User): void {
    state.users.push(new_user);
  },
  SET_CAN_BID(state, can_bid: boolean): void {
    state.can_bid = can_bid;
  },
  SET_CAN_POST_PLANNING(state, can_post_planning: boolean): void {
    state.can_post_planning = can_post_planning;
  },
  SET_CLEARING_AVAILABLE(state, clearing_available: boolean): void {
    //console.log("clearing", state.clearing_available, clearing_available);
    state.clearing_available = clearing_available;
  },
  SET_RESULTS_AVAILABLE(state, results_available: boolean): void {
    state.results_available = results_available;
  },
  SET_PHASE_INFOS(state, phase_infos): void {
    Vue.set(state, "phase_infos", {
      phase_no: phase_infos.phase_no,
      nb_phases: phase_infos.nb_phases,
      start_time: new Date(phase_infos.start_time),
      clearing_time: new Date(phase_infos.clearing_time),
      planning_time: new Date(phase_infos.planning_time),
    });
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
  session_nb_users(state): number {
    return state.users.length;
  },
  can_bid(state): boolean {
    return state.can_bid;
  },
  can_post_planning(state): boolean {
    return state.can_post_planning;
  },
  clearing_available(state): boolean {
    return state.clearing_available;
  },
  results_available(state): boolean {
    return state.results_available;
  },
  phase_infos(state): Session["phase_infos"] {
    return state.phase_infos;
  },
  nb_phases(state): number {
    return state.phase_infos ? state.phase_infos.nb_phases : -1;
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
