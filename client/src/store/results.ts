import Vuex, { Module, GetterTree, MutationTree, ActionTree } from "vuex";
import Vue from "vue";
import { RootState } from "./index";

interface PlantDispatch {
  phase_no: number;
  plant_id: string;
  p_dispatch_mw: number;
  stock_start_mwh: number;
  stock_end_mwh: number;
  type: "nuc" | "therm" | "hydro" | "ren" | "storage";
}

export interface PhasePlanning {
  phase_no: number;
  planning: PlantDispatch[];
}

export interface ResultsPhase {
  phase_no: number;
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

export interface ResultsState extends ResultsPhase {
  rankings: {
    phase: { username: string; rank: number; balance: number }[];
    overall: { username: string; rank: number }[];
  };
  previous_plannings: PlantDispatch[];
  previous_results: ResultsPhase[];
}

// ------------------------ STATE -------------------------
export const state: ResultsState = {
  phase_no: -1,
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
  rankings: {
    phase: [],
    overall: [],
  },
  previous_plannings: [],
  previous_results: [],
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<ResultsState, RootState> = {
  async loadResultsContent({ commit, rootState }): Promise<void> {
    const api_url = rootState.api_url;
    const session_id = rootState.session.id;
    const user_id = rootState.user.user_id;
    let data = {
      current_results: {},
      previous_results: {},
      previous_plannings: {},
    };
    const res = await fetch(
      `${api_url}/session/${session_id}/user/${user_id}/results`,
      {
        method: "GET",
      }
    );
    if (res.status === 200) {
      data = await res.json();
    } else {
      console.log(await res.text());
    }
    commit("SET_RESULTS", data.current_results);
    commit("SET_PREVIOUS_RESULTS", data.previous_results);
    commit("SET_PREVIOUS_PLANNINGS", data.previous_plannings);
    console.log(data.previous_results);
  },
  async loadRankings({ commit, rootState }): Promise<void> {
    const api_url = rootState.api_url;
    const session_id = rootState.session.id;
    let results = {};

    const res = await fetch(`${api_url}/session/${session_id}/rankings`, {
      method: "GET",
    });
    if (res.status === 200) {
      results = await res.json();
    } else {
      console.log(await res.text());
    }
    commit("SET_RANKINGS", results);
  },
};

// ------------------------ MUTATIONS -------------------------
export const mutations: MutationTree<ResultsState> = {
  SET_RESULTS(state, results: ResultsState): void {
    Object.entries(results).forEach(([k, v]) => {
      Vue.set(state, k, v);
    });
  },
  SET_PREVIOUS_RESULTS(state, previous_results: ResultsPhase[]): void {
    state.previous_results = previous_results.sort(
      (a, b) => a.phase_no - b.phase_no
    );
  },
  SET_PREVIOUS_PLANNINGS(state, previous_plannings: PlantDispatch[]): void {
    state.previous_plannings = previous_plannings;
  },
  SET_RANKINGS(state, rankings: ResultsState["rankings"]): void {
    state.rankings.phase = rankings.phase;
    state.rankings.overall = rankings.overall;
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
  previous_plannings(state): PhasePlanning[] {
    const res: { phase_no: number; planning: PlantDispatch[] }[] = [];
    for (let i = 0; i < state.previous_plannings.length; i++) {
      const idx = res.findIndex(
        (r) => r.phase_no === state.previous_plannings[i].phase_no
      );
      if (idx === -1) {
        res.push({
          phase_no: state.previous_plannings[i].phase_no,
          planning: [state.previous_plannings[i]],
        });
      } else {
        res[idx].planning.push(state.previous_plannings[i]);
      }
    }
    return res.sort((a, b) => a.phase_no - b.phase_no);
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
