import Vuex, { Module, GetterTree, MutationTree, ActionTree } from "vuex";
import { RootState } from "./index";

export interface Auction {
  name: string;
  id: string;
  status: "Open" | "Running" | "Close";
}

export interface AuctionState extends Auction {}

// ------------------------ STATE -------------------------
export const state: AuctionState = {
  name: "",
  id: "",
  status: "Close",
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<AuctionState, RootState> = {
  setAuction({ commit }, payload: Auction): void {
    commit("SET_AUCTION", payload);
  },
};

// ------------------------ MUTATIONS -------------------------
export const mutations: MutationTree<AuctionState> = {
  SET_AUCTION(state, payload: Auction): void {
    state = payload;
  },
};

// ------------------------ GETTERS -------------------------
export const getters: GetterTree<AuctionState, RootState> = {
  auction(state): Auction {
    return state;
  },
  auction_id(state): string {
    return state.id;
  },
  auction_name(state): string {
    return state.name;
  },
};

// ------------------------ MODULE -------------------------
export const auction: Module<AuctionState, RootState> = {
  namespaced: true,
  state,
  getters,
  actions,
  mutations,
};
