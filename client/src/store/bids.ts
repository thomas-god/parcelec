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
  energy_exchanges: {
    buy: EnergyExchange;
    sell: EnergyExchange;
  };
}

// ------------------------ STATE -------------------------
export const state: BidsState = {
  bids: [],
  energy_exchanges: {
    sell: {
      volume_mwh: 0,
      price_eur_per_mwh: 0,
    },
    buy: {
      volume_mwh: 0,
      price_eur_per_mwh: 0,
    },
  },
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<BidsState, RootState> = {
  async loadBidsContent({ commit, rootState }): Promise<void> {
    const api_url = rootState.api_url;
    const session_id = rootState.session.id;
    const user_id = rootState.user.user_id;
    const bids = await (
      await fetch(`${api_url}/session/${session_id}/user/${user_id}/bids`, {
        method: "GET",
      })
    ).json();
    commit("SET_BIDS", bids);
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
