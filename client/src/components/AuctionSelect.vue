<template>
  <div>
    <h2>Choisissez une partie à rejoindre</h2>
    <ul class="auctions_list">
      <li
        v-for="a in open_auctions"
        :key="a.name"
        @click="setAuction({ ...a, status: 'Open' })"
      >
        <span>{{ a.name }}</span>
      </li>
      <li v-if="open_auctions.length === 0">
        Il n'y a pas de partie à rejoindre
      </li>
    </ul>
    <div v-if="allow_new_auction" class="auction_open">
      <label for="auction_open_input">
        Ou bien entrez le nom d'une nouvelle partie :
      </label>
      <div>
        <input
          type="text"
          v-model="new_auction_name"
          v-on:keyup.enter="openAuction()"
          id="auction_open_input"
        />
        <button @click="openAuction()">Open</button>
      </div>
      <span v-if="new_auction_name_err" style="color: red">{{
        new_auction_name_err_msg
      }}</span>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Auction } from "../store/auction";

const auctionModule = namespace("auction");

@Component
export default class User extends Vue {
  @Prop({ default: false }) allow_new_auction!: boolean;
  @auctionModule.Getter auction!: Auction;
  @auctionModule.Action setAuction!: (payload: Auction) => void;

  // List of open auctions
  open_auctions: Auction[] = [];
  async getOpenAuctions(): Promise<void> {
    const res = await fetch("http://localhost:3000/auction/list_open", {
      method: "GET",
    });
    this.open_auctions = await res.json();
  }

  // Open a new session
  new_auction_name = "";
  new_auction_name_err = false;
  new_auction_name_err_msg = "";
  async openAuction() {
    const res = await fetch("http://localhost:3000/auction/open", {
      method: "PUT",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ auction_name: this.new_auction_name }),
    });
    if (res.status === 200) {
      this.new_auction_name_err = false;
      this.new_auction_name_err_msg = "";
    } else {
      this.new_auction_name_err = true;
      this.new_auction_name_err_msg = await res.text();
    }
  }

  async created(): Promise<void> {
    await this.getOpenAuctions();
  }
}
</script>

<style scoped>
.auctions_list {
  max-width: 200px;
  margin: auto;
  margin-bottom: 1.5rem;
  padding: 1rem;
  border: 1px solid rgba(0, 0, 0, 0.493);
  border-radius: 3px;
  box-shadow: 12px 12px 2px 1px rgba(28, 28, 56, 0.26);
}

ul {
  list-style-type: none;
  padding: 0;
}

li span {
  padding: 0 0.5rem;
}

li:hover span {
  background-color: rgb(0, 151, 98);
}

.auction_open {
  display: flex;
  flex-direction: column;
  margin: auto;
}
</style>
