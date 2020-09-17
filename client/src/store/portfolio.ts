import Vuex, { Module, GetterTree, MutationTree, ActionTree } from "vuex";
import Vue from "vue";
import { RootState } from "./index";

export interface PowerPlant {
  id: string;
  type: "nuc" | "therm" | "hydro" | "ren" | "storage";
  p_min_mw: number;
  p_max_mw: number;
  stock_max_mwh: number;
  price_eur_per_mwh: number;
}

export interface Portfolio {
  power_plants: PowerPlant[];
}

export interface PortfolioState extends Portfolio {}

// ------------------------ STATE -------------------------
export const state: PortfolioState = {
  power_plants: [],
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<PortfolioState, RootState> = {
  /**
   * Load portfolio related data (power plants, conso forecast) into
   * the store.
   */
  async loadPortfolioContent({ commit, rootState }): Promise<void> {
    const power_plants = await getPowerPlants(
      rootState.session.id,
      rootState.user.user_id
    );
    commit("SET_POWER_PLANTS", power_plants);
  },
};

// ------------------------ MUTATIONS -------------------------
export const mutations: MutationTree<PortfolioState> = {
  SET_POWER_PLANTS(state, power_plants: PowerPlant[]): void {
    state.power_plants = power_plants;
  },
};

// ------------------------ GETTERS -------------------------
export const getters: GetterTree<PortfolioState, RootState> = {
  power_plants(state): PowerPlant[] {
    return state.power_plants;
  },
};

// ------------------------ MODULE -------------------------
export const portfolio: Module<PortfolioState, RootState> = {
  namespaced: true,
  state,
  getters,
  actions,
  mutations,
};

// ------------------------ Helper functions ---------------
async function getPowerPlants(
  session_id: string,
  user_id: string
): Promise<PowerPlant[]> {
  const res = await fetch(
    `${process.env.VUE_APP_API_URL}/session/${session_id}/user/${user_id}/portfolio`,
    {
      method: "GET",
    }
  );
  if (res.status === 200) {
    const body = await res.json();
    return body as PowerPlant[];
  } else {
    return [];
  }
}
