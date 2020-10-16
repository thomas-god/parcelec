import Vuex, { Module, GetterTree, MutationTree, ActionTree } from "vuex";
import { RootState } from "./index";

export interface OTC {
  type: "buy" | "sell";
  volume_mwh: number;
  price_eur_per_mwh: number;
  id: string;
}

export interface EnergyExchange {
  volume_mwh: number;
  price_eur_per_mwh: number;
}

export interface OTCState {
  otcs: OTC[];
}

// ------------------------ STATE -------------------------
export const state: OTCState = {
  otcs: [],
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<OTCState, RootState> = {
  async loadOTCsContent({ commit, rootState }): Promise<void> {
    const api_url = rootState.api_url;
    const session_id = rootState.session.id;
    const user_id = rootState.user.user_id;
    let otcs = [];
    const res = await await fetch(
      `${api_url}/session/${session_id}/user/${user_id}/otc`,
      {
        method: "GET",
      }
    );
    if (res.status === 200) {
      otcs = await res.json();
    } else {
      console.log(await res.text());
    }
    commit("SET_OTCS", otcs);
  },
};

// ------------------------ MUTATIONS -------------------------
export const mutations: MutationTree<OTCState> = {
  SET_OTCS(state, otcs: OTC[]): void {
    state.otcs = otcs;
  },
  PUSH_OTC(state, otc: OTC): void {
    state.otcs.push(otc);
  },
};

// ------------------------ GETTERS -------------------------
export const getters: GetterTree<OTCState, RootState> = {};

// ------------------------ MODULE -------------------------
export const otcs: Module<OTCState, RootState> = {
  namespaced: true,
  state,
  getters,
  actions,
  mutations,
};
