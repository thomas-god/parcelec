import Vuex, { Module, GetterTree, MutationTree, ActionTree } from "vuex";
import { RootState } from "./index";

export interface Auction {
  name: string;
  id: string;
  status: "Open" | "Running" | "Close";
  users: string[];
}

export interface AuctionState extends Auction {}

// ------------------------ STATE -------------------------
export const state: AuctionState = {
  name: "",
  id: "",
  status: "Close",
  users: [],
};

// ------------------------ ACTIONS -------------------------
export const actions: ActionTree<AuctionState, RootState> = {
  async setAuction({ commit }, payload: Auction): Promise<void> {
    const users = await getUsersList(payload.id);
    commit("SET_AUCTION", { ...payload, users });
  },
  async updateUsersList({ state, commit }): Promise<void> {
    const users = await getUsersList(state.id);
    commit("SET_USERS", users);
  },
};

// ------------------------ MUTATIONS -------------------------
export const mutations: MutationTree<AuctionState> = {
  SET_AUCTION(state, auction: Auction): void {
    state.id = auction.id;
    state.name = auction.name;
    state.status = auction.status;
    state.users = auction.users;
  },
  SET_USERS(state, users: string[]): void {
    state.users = users;
  },
  PUSH_NEW_USER(state, new_user: string): void {
    state.users.push(new_user);
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

// ------------------------ Helper functions ---------------
async function getUsersList(auction_id: string): Promise<string[]> {
  const res = await fetch(`http://localhost:3000/auction/${auction_id}`, {
    method: "GET",
  });
  if (res.status === 200) {
    const body = await res.json();
    return body.users as string[];
  } else {
    return [];
  }
}
