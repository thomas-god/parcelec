import Vuex, { Module, GetterTree, MutationTree, ActionTree } from "vuex";
import { RootState } from "./index";

export interface Bid {
  type: "buy" | "sell";
  volume_mwh: number;
  price_eur_per_mwh: number;
  id: string;
}

export interface EnergyExchange {
  volume_mwh: number;
  price_eur_per_mwh: number;
}

export interface BidsState {
  bids: Bid[];
  clearing: EnergyExchange;
  sell: EnergyExchange;
  buy: EnergyExchange;
}

// ------------------------ STATE -------------------------
export const state: BidsState = {
  bids: [],
  clearing: { volume_mwh: 0, price_eur_per_mwh: 0 },
  sell: { volume_mwh: 0, price_eur_per_mwh: 0 },
  buy: { volume_mwh: 0, price_eur_per_mwh: 0 },
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<BidsState, RootState> = {
  async loadBidsContent({ commit, rootState }): Promise<void> {
    const api_url = rootState.api_url;
    const session_id = rootState.session.id;
    const user_id = rootState.user.user_id;
    let bids = [];
    const res = await await fetch(
      `${api_url}/session/${session_id}/user/${user_id}/bids`,
      {
        method: "GET",
      }
    );
    if (res.status === 200) {
      bids = await res.json();
    } else {
      console.log(await res.text());
    }
    commit("SET_BIDS", bids);
  },
  async loadClearingContent({ commit, rootState }): Promise<void> {
    const api_url = rootState.api_url;
    const session_id = rootState.session.id;
    const user_id = rootState.user.user_id;

    // Clearing content
    let clearing = { volume_mwh: 0, price_eur_per_mwh: 0 };
    const res_clearing = await fetch(
      `${api_url}/session/${session_id}/clearing`,
      {
        method: "GET",
      }
    );
    if (res_clearing.status === 200) {
      clearing = await res_clearing.json();
    }
    commit("SET_CLEARING", clearing);

    // Energy exchange content
    let exchanges = [];
    const res_exchanges = await fetch(
      `${api_url}/session/${session_id}/user/${user_id}/clearing`,
      {
        method: "GET",
      }
    );
    if (res_exchanges.status === 200) {
      exchanges = await res_exchanges.json();
    }
    commit("SET_ENERGY_EXCHANGES", exchanges);
  },
};

// ------------------------ MUTATIONS -------------------------
export const mutations: MutationTree<BidsState> = {
  SET_BIDS(state, bids: Bid[]): void {
    state.bids = bids;
  },
  PUSH_BID(state, bid: Bid): void {
    state.bids.push(bid);
  },
  DELETE_BID(state, bid_id: string): void {
    const id = state.bids.findIndex((bid) => bid.id === bid_id);
    state.bids.splice(id, 1);
  },
  SET_CLEARING(state, clearing: EnergyExchange): void {
    state.clearing = clearing;
  },
  SET_ENERGY_EXCHANGES(state, energy_exchanges: any[]): void {
    // Set buy
    const buy = energy_exchanges.find((e) => e.type === "buy");
    if (buy !== undefined) {
      state.buy = {
        volume_mwh: buy.volume_mwh,
        price_eur_per_mwh: buy.price_eur_per_mwh,
      };
    } else {
      state.buy = { volume_mwh: 0, price_eur_per_mwh: 0 };
    }

    // Set sell
    const sell = energy_exchanges.find((e) => e.type === "sell");
    if (sell !== undefined) {
      state.sell = {
        volume_mwh: sell.volume_mwh,
        price_eur_per_mwh: sell.price_eur_per_mwh,
      };
    } else {
      state.sell = { volume_mwh: 0, price_eur_per_mwh: 0 };
    }
  },
};

// ------------------------ GETTERS -------------------------
export const getters: GetterTree<BidsState, RootState> = {
  bids(state): Bid[] {
    return state.bids;
  },
};

// ------------------------ MODULE -------------------------
export const bids: Module<BidsState, RootState> = {
  namespaced: true,
  state,
  getters,
  actions,
  mutations,
};
