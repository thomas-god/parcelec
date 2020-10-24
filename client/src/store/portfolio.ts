import Vuex, {
  Module,
  GetterTree,
  MutationTree,
  ActionTree,
  Commit,
} from "vuex";
import { RootState } from "./index";

export interface PowerPlant {
  id: string;
  type: "nuc" | "therm" | "hydro" | "ren" | "storage";
  p_min_mw: number;
  p_max_mw: number;
  stock_max_mwh: number;
  price_eur_per_mwh: number;
  planning: number;
  planning_modif: number;
  stock_mwh: number;
}

export interface Portfolio {
  power_plants: PowerPlant[];
  conso: number;
}

export interface PortfolioState extends Portfolio {}

// ------------------------ STATE -------------------------
export const state: PortfolioState = {
  power_plants: [],
  conso: 0,
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<PortfolioState, RootState> = {
  /**
   * Load portfolio related data (power plants, conso forecast) into
   * the store.
   */
  async loadPortfolioContent({ commit, rootState }): Promise<void> {
    loadPowerPlants(commit, rootState.session.id, rootState.user.user_id);
    loadConsoForecast(commit, rootState.session.id, rootState.user.user_id);
  },
  resetPlanning({ state }): void {
    state.power_plants.forEach((pp) => {
      pp.planning_modif = pp.planning;
    });
  },
  onSuccessfulPlanningUpdate(): void {
    state.power_plants.forEach((pp) => {
      pp.planning = pp.planning_modif;
    });
  },
};

// ------------------------ MUTATIONS -------------------------
export const mutations: MutationTree<PortfolioState> = {
  SET_POWER_PLANTS(state, power_plants: PowerPlant[]): void {
    state.power_plants = power_plants.map((pp) => {
      return {
        ...pp,
        planning_modif: pp.planning,
      };
    });
  },
  SET_CONSO(state, conso: number): void {
    state.conso = conso;
  },
};

// ------------------------ GETTERS -------------------------
export const getters: GetterTree<PortfolioState, RootState> = {
  power_plants(state): PowerPlant[] {
    return state.power_plants;
  },
  conso(state): number {
    return state.conso;
  },
  delta_planning(state): number {
    return state.power_plants
      .map((pp) => pp.planning - pp.planning_modif)
      .reduce((s, c) => s + c, 0);
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
async function loadPowerPlants(
  commit: Commit,
  session_id: string,
  user_id: string
): Promise<void> {
  let power_plants = [];
  const res = await fetch(
    `${process.env.VUE_APP_API_URL}/session/${session_id}/user/${user_id}/portfolio`,
    {
      method: "GET",
    }
  );
  if (res.status === 200) {
    power_plants = await res.json();
  }
  commit("SET_POWER_PLANTS", power_plants);
}

async function loadConsoForecast(
  commit: Commit,
  session_id: string,
  user_id: string
): Promise<void> {
  let conso = 0;
  const res = await fetch(
    `${process.env.VUE_APP_API_URL}/session/${session_id}/user/${user_id}/conso`,
    {
      method: "GET",
    }
  );
  if (res.status === 200) {
    conso = (await res.json()).conso_mw;
  }
  commit("SET_CONSO", conso);
}
