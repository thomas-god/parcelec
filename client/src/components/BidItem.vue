<template>
  <div class="bid">
    <span
      >Vous {{ bid.type === "sell" ? "vendez" : "achetez" }}
      <strong>{{ Math.floor(bid.volume_mwh).toLocaleString("fr-FR") }}</strong>
      MWh à
      <strong>{{
        Math.floor(bid.price_eur_per_mwh).toLocaleString("fr-FR")
      }}</strong>
      €/MWh</span
    >
    <button @click="deleteBid" :disabled="!dummy && !can_bid">🗑</button>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue, Watch } from "vue-property-decorator";
import { use } from "vue/types/umd";
import { State, Action, Getter, namespace } from "vuex-class";
import { Bid } from "../store/bids";
import { v4 as uuid } from "uuid";

const sessionModule = namespace("session");
const userModule = namespace("user");
const bidsModule = namespace("bids");

@Component
export default class BidItem extends Vue {
  @Prop() edit!: boolean;
  @Prop() bid!: Bid;
  @Prop({ default: false }) dummy!: boolean;
  @sessionModule.Getter can_bid!: boolean;
  @State api_url!: string;
  @sessionModule.Getter session_id!: string;
  @userModule.Getter user_id!: string;
  @bidsModule.Mutation DELETE_BID!: (bid_id: string) => void;
  async deleteBid(): Promise<void> {
    if (!this.dummy) {
      const res = await fetch(
        `${this.api_url}/session/${this.session_id}/user/${this.user_id}/bid/${this.bid.id}`,
        {
          method: "DELETE"
        }
      );
      if (res.status === 200) {
        this.DELETE_BID(this.bid.id);
      } else {
        console.log(await res.text());
      }
    } else {
      this.DELETE_BID(this.bid.id);
    }
  }
}
</script>

<style scoped>
.bid {
  font-size: 1rem;
  padding: 0.4rem;
}

.bid strong {
  padding-left: 0.7ch;
  padding-right: 0.4ch;
}

.bid button {
  border: none;
  background-color: white;
}
</style>
