import Vuex, { Module, GetterTree, MutationTree, ActionTree } from "vuex";
import Vue from "vue";
import { RootState } from "./index";

export interface ResultsState {
  conso_mwh: number;
  conso_eur: number;
  prod_mwh: number;
  prod_eur: number;
  sell_mwh: number;
  sell_eur: number;
  buy_mwh: number;
  buy_eur: number;
  imbalance_mwh: number;
  imbalance_costs_eur: number;
  balance_eur: number;
  ranking_current: number;
  ranking_overall: number;
}

// ------------------------ STATE -------------------------
export const state: ResultsState = {
  conso_mwh: 0,
  conso_eur: 0,
  prod_mwh: 0,
  prod_eur: 0,
  sell_mwh: 0,
  sell_eur: 0,
  buy_mwh: 0,
  buy_eur: 0,
  imbalance_mwh: 0,
  imbalance_costs_eur: 0,
  balance_eur: 0,
  ranking_current: 0,
  ranking_overall: 0,
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<ResultsState, RootState> = {
  async loadResultsContent({ commit, rootState }): Promise<void> {
    const api_url = rootState.api_url;
    const session_id = rootState.session.id;
    const user_id = rootState.user.user_id;
    let results = {};
    const res = await await fetch(
      `${api_url}/session/${session_id}/user/${user_id}/results`,
      {
        method: "GET",
      }
    );
    if (res.status === 200) {
      results = await res.json();
    } else {
      console.log(await res.text());
    }
    commit("SET_RESULTS", results);
  },
};

// ------------------------ MUTATIONS -------------------------
export const mutations: MutationTree<ResultsState> = {
  SET_RESULTS(state, results: ResultsState): void {
    Object.entries(results).forEach(([k, v]) => {
      Vue.set(state, k, v);
    });
  },
};

// ------------------------ GETTERS -------------------------
export const getters: GetterTree<ResultsState, RootState> = {
  user_rankings(state): { current: number; overall: number } {
    return {
      current: state.ranking_current,
      overall: state.ranking_overall,
    };
  },
};

// ------------------------ MODULE -------------------------
export const results: Module<ResultsState, RootState> = {
  namespaced: true,
  state,
  getters,
  actions,
  mutations,
};
