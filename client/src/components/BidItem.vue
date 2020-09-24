<template>
  <div class="bid__box" :style="borderStyle">
    <div class="bid__input" v-if="edit">
      <strong style="grid-area: type;">{{ actionString }} : </strong>
      <span style="grid-area: inputs;">
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
      </span>
      <button
        @click="postBid"
        :disabled="!dummy && !can_bid"
        style="grid-area: btn;"
      >
        ➕
      </button>
      <span class="bid__error" style="grid-area: err">{{
        inputs_err_msg.volume === ""
          ? inputs_err_msg.price
          : inputs_err_msg.volume
      }}</span>
    </div>
    <span v-else class="bids__list">
      <span class="bids__list-puce">📋</span>
      {{ `${bid.volume_mwh} MWh, à ${bid.price_eur_per_mwh} €/MWh` }}
      <button @click="deleteBid" :disabled="!dummy && !can_bid">🗑</button>
    </span>
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
  @Prop() type!: "buy" | "sell";
  @Prop() edit!: boolean;
  @Prop() bid!: Bid;
  @Prop({ default: false }) dummy!: boolean;
  @sessionModule.Getter can_bid!: boolean;
  volume_mwh = 0;
  price_eur_per_mwh = 0;
  inputs_err_msg = {
    price: "",
    volume: ""
  };

  validateInputs(): boolean {
    return this.validateVolume() && this.validatePrice();
  }

  @Watch("volume_mwh")
  validateVolume(): boolean {
    let flag = true;
    console.log("checking volume");
    if (isNaN(Number(this.volume_mwh))) {
      flag = false;
      this.inputs_err_msg.volume = "Le volume doit être un nombre";
    } else if (this.volume_mwh <= 0) {
      flag = false;
      this.inputs_err_msg.volume = "Le volume doit être positif";
    } else {
      this.inputs_err_msg.volume = "";
    }
    return flag;
  }

  @Watch("price_eur_per_mwh")
  validatePrice(): boolean {
    console.log(
      "checking price",
      this.price_eur_per_mwh,
      isNaN(Number(this.price_eur_per_mwh))
    );
    let flag = true;
    if (isNaN(Number(this.price_eur_per_mwh))) {
      console.log("price false");

      flag = false;
      this.inputs_err_msg.price = "Le prix doit être un nombre";
    } else {
      this.inputs_err_msg.price = "";
    }
    return flag;
  }

  @State api_url!: string;
  @sessionModule.Getter session_id!: string;
  @userModule.Getter user_id!: string;
  @bidsModule.Mutation PUSH_BID!: (bid: Bid) => void;
  async postBid(): Promise<void> {
    if (!this.dummy) {
      const res = await fetch(
        `${this.api_url}/session/${this.session_id}/user/${this.user_id}/bid`,
        {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            bid: {
              type: this.type,
              volume_mwh: this.volume_mwh,
              price_eur_per_mwh: this.price_eur_per_mwh
            }
          })
        }
      );
      if (res.status === 201) {
        const bid_id = (await res.json()).bid_id;
        this.PUSH_BID({
          type: this.type,
          volume_mwh: this.volume_mwh,
          price_eur_per_mwh: this.price_eur_per_mwh,
          id: bid_id
        });
      } else {
        console.log(await res.text());
      }
    } else {
      this.PUSH_BID({
        type: this.type,
        volume_mwh: this.volume_mwh,
        price_eur_per_mwh: this.price_eur_per_mwh,
        id: uuid()
      });
    }
  }

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

@media screen and (min-width: 500px) {
  .bid__input {
    display: grid;
    grid-template-areas:
      "type inputs btn"
      "X err err";
    grid-template-columns: 70px auto auto;
    grid-template-rows: auto 30px;
    align-content: center;
  }
}

@media screen and (max-width: 500px) {
  .bid__input {
    display: grid;
    grid-template-areas:
      "type X X"
      "inputs inputs btn"
      "err err err";
    grid-template-columns: 70px auto 20px;
    grid-template-rows: 30px auto 30px;
    align-content: center;
  }
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
