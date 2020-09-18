<template>
  <div class="bid__box" :style="borderStyle">
    <span class="bid__input" v-if="edit && can_bid">
      <strong>{{ actionString }} : </strong>
      <input
        type="number"
        min="0"
        class="bid__number_input"
        v-model="volume_mwh"
      />
      <span> MWh à </span>
      <input
        type="number"
        class="bid__number_input"
        v-model="price_eur_per_mwh"
      />
      <span> €/MWh </span>
      <button @click="postBid" :disabled="!can_bid">➕</button>
    </span>
    <span v-else class="bids__list">
      <span class="bids__list-puce">📋</span>
      {{ `${bid.volume_mwh} MWh, à ${bid.price_eur_per_mwh} €/MWh` }}
      <button @click="deleteBid" :disabled="!can_bid">🗑</button>
    </span>
    <span class="bid__error">{{ volume_mwh_err_msg }}</span>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue, Watch } from "vue-property-decorator";
import { use } from "vue/types/umd";
import { State, Action, Getter, namespace } from "vuex-class";
import { Bid } from "../store/bids";

const sessionModule = namespace("session");
const userModule = namespace("user");
const bidsModule = namespace("bids");

@Component
export default class BidItem extends Vue {
  @Prop() type!: "buy" | "sell";
  @Prop() edit!: boolean;
  @Prop() bid!: Bid;
  @sessionModule.Getter can_bid!: boolean;
  volume_mwh = 0;
  volume_mwh_err_msg = "";
  price_eur_per_mwh = 0;

  validateInputs(): boolean {
    return this.validateVolume() && this.validatePrice();
  }

  @Watch("volume_mwh")
  validateVolume(): boolean {
    console.log(this.volume_mwh_err_msg);
    let flag = true;
    if (isNaN(Number(this.volume_mwh))) {
      flag = false;
      this.volume_mwh_err_msg = "Le volume doit être un nombre";
    } else if (this.volume_mwh <= 0) {
      flag = false;
      this.volume_mwh_err_msg = "Le volume doit être positif";
    }
    return flag;
  }
  validatePrice(): boolean {
    let flag = true;
    if (isNaN(Number(this.price_eur_per_mwh))) {
      flag = false;
      this.volume_mwh_err_msg = "Le volume doit être un nombre";
    }
    return flag;
  }

  @State api_url!: string;
  @sessionModule.Getter session_id!: string;
  @userModule.Getter user_id!: string;
  @bidsModule.Mutation PUSH_BID!: (bid: Bid) => void;
  async postBid(): Promise<void> {
    const res = await fetch(
      `${this.api_url}/session/${this.session_id}/user/${this.user_id}/bid`,
      {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          bid: {
            type: this.type,
            volume_mwh: this.volume_mwh,
            price_eur_per_mwh: this.price_eur_per_mwh,
          },
        }),
      }
    );
    if (res.status === 201) {
      const bid_id = (await res.json()).bid_id;
      this.PUSH_BID({
        type: this.type,
        volume_mwh: this.volume_mwh,
        price_eur_per_mwh: this.price_eur_per_mwh,
        id: bid_id,
      });
    } else {
      console.log(await res.text());
    }
  }

  @bidsModule.Mutation DELETE_BID!: (bid_id: string) => void;
  async deleteBid(): Promise<void> {
    const res = await fetch(
      `${this.api_url}/session/${this.session_id}/user/${this.user_id}/bid/${this.bid.id}`,
      {
        method: "DELETE",
      }
    );
    if (res.status === 200) {
      this.DELETE_BID(this.bid.id);
    } else {
      console.log(await res.text());
    }
  }

  get actionString(): string {
    return this.type === "buy" ? "Acheter" : "Vendre";
  }

  get borderStyle(): string {
    return ""; /* this.edit
      ? `border: 2px solid ${this.type === "sell" ? "red" : "green"};`
      : ""; */
  }
}
</script>

<style scoped>
/* Chrome, Safari, Edge, Opera */
input::-webkit-outer-spin-button,
input::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
/* Firefox */
input[type="number"] {
  -moz-appearance: textfield;
}
.bid__box {
  border-radius: 3px;
  margin: 0.7rem;
}

.bid__input {
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-around;
}

button {
  padding: 0;
  margin: auto 10px;
  border: none;
  border-radius: 2px;
  background: none;
  font-size: 1rem;
}
.bid__number_input {
  border: none;
  border-bottom: 2px solid gray;
  width: 50px;
  font-size: 1rem;
  text-align: center;
}

.bid__error {
  font-size: 0.9rem;
  color: red;
}

.bids__list {
  display: flex;
  flex-direction: row;
  align-items: center;
}

.bids__list * {
  padding: 0 10px;
}
.bids__list-puce {
  padding-left: 15px;
  font-size: 1.2rem;
}
</style>
