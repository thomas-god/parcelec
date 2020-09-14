import Vuex, { Module, GetterTree, MutationTree, ActionTree } from "vuex";
import { RootState } from "./index";

export interface User {
  name: string;
  ready: boolean;
}

export interface Auction {
  name: string;
  id: string;
  status: "Open" | "Running" | "Close";
  users: User[];
  can_bid: boolean;
}

export interface AuctionState extends Auction {}

// ------------------------ STATE -------------------------
export const state: AuctionState = {
  name: "",
  id: "",
  status: "Open",
  users: [],
  can_bid: false,
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
  setStatus({ state, commit }, status: Auction["status"]): void {
    if (state.status === "Open" && status === "Running") {
      commit("UPDATE_CAN_BID", true);
    }
    commit("UPDATE_STATUS", status);
  },
  updateBidAbility({ commit }, bid_ability: boolean): void {
    commit("UPDATE_CAN_BID", bid_ability);
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
  SET_USERS(state, users: User[]): void {
    state.users = users;
  },
  PUSH_NEW_USER(state, new_user: User): void {
    state.users.push(new_user);
  },
  UPDATE_STATUS(state, status: Auction["status"]): void {
    state.status = status;
  },
  UPDATE_CAN_BID(state, can_bid: boolean): void {
    state.can_bid = can_bid;
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
  auction_status(state): string {
    return state.status;
  },
  can_bid(state): boolean {
    return state.can_bid;
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
async function getUsersList(auction_id: string): Promise<User[]> {
  const res = await fetch(`http://localhost:3000/auction/${auction_id}`, {
    method: "GET",
  });
  if (res.status === 200) {
    const body = await res.json();
    return body.users as User[];
  } else {
    return [];
  }
}
