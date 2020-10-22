<template>
  <div class="bid__container">
    <div class="bid__container_qte_txt">Quantité (MWh)</div>
    <div class="bid__container_qte_input">
      <input type="number" min="0" step="0.01" v-model="volume_mwh" />
    </div>
    <div class="bid__container_qte_err">
      {{ show_err_msg ? inputs_err_msg_volume : "" }}
    </div>
    <div class="bid__container_price_txt">Prix (€/MWh)</div>
    <div class="bid__container_price_input">
      <input type="number" step="0.01" v-model="price_eur_per_mwh" />
    </div>
    <div class="bid__container_price_err">
      {{ show_err_msg ? inputs_err_msg_price : "" }}
    </div>
    <div class="bid__container_actions">
      <Btn @click="postBid('buy')" background_color="green">Acheter</Btn>
      <Btn @click="postBid('sell')" background_color="green">Vendre</Btn>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue, Watch } from "vue-property-decorator";
import { use } from "vue/types/umd";
import { State, Action, Getter, namespace } from "vuex-class";
import { Bid } from "../store/bids";
import { v4 as uuid } from "uuid";
import Btn from "./base/Button.vue";

const sessionModule = namespace("session");
const userModule = namespace("user");
const bidsModule = namespace("bids");

@Component({ components: { Btn } })
export default class BidItem extends Vue {
  @Prop() edit!: boolean;
  @Prop({ default: false }) dummy!: boolean;
  @sessionModule.Getter can_bid!: boolean;
  volume_mwh: number | "" = 0;
  price_eur_per_mwh: number | "" = 0;
  inputs_err_msg_volume = "";
  inputs_err_msg_price = "";
  show_err_msg = true;

  validateInputs(): boolean {
    return this.validateVolume() && this.validatePrice();
  }

  @Watch("volume_mwh")
  validateVolume(): boolean {
    let flag = true;
    if (this.show_err_msg) {
      if (isNaN(Number(this.volume_mwh)) || this.volume_mwh === "") {
        flag = false;
        this.inputs_err_msg_volume = "Le volume doit être un nombre";
      } else if (this.volume_mwh <= 0) {
        flag = false;
        this.inputs_err_msg_volume = "Le volume doit être positif";
      } else {
        this.inputs_err_msg_volume = "";
      }
    }
    return flag;
  }

  @Watch("price_eur_per_mwh")
  validatePrice(): boolean {
    let flag = true;
    if (this.show_err_msg) {
      if (
        isNaN(Number(this.price_eur_per_mwh)) ||
        this.price_eur_per_mwh === ""
      ) {
        flag = false;
        this.inputs_err_msg_price = "Le prix doit être un nombre";
      } else {
        this.inputs_err_msg_price = "";
      }
    }
    return flag;
  }

  @State api_url!: string;
  @sessionModule.Getter session_id!: string;
  @userModule.Getter user_id!: string;
  @bidsModule.Mutation PUSH_BID!: (bid: Bid) => void;
  async postBid(type: "sell" | "buy"): Promise<void> {
    if (!this.dummy && this.validateInputs()) {
      const res = await fetch(
        `${this.api_url}/session/${this.session_id}/user/${this.user_id}/bid`,
        {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            bid: {
              type: type,
              volume_mwh: Number(this.volume_mwh),
              price_eur_per_mwh: Number(this.price_eur_per_mwh)
            }
          })
        }
      );
      if (res.status === 201) {
        const bid_id = (await res.json()).bid_id;
        this.PUSH_BID({
          type: type,
          volume_mwh: Number(this.volume_mwh),
          price_eur_per_mwh: Number(this.price_eur_per_mwh),
          id: bid_id
        });
        this.hideErrMsg();
        this.volume_mwh = 0;
        this.price_eur_per_mwh = 0;
      } else {
        console.log(await res.text());
      }
    } else {
      if (this.validateInputs()) {
        this.PUSH_BID({
          type: type,
          volume_mwh: Number(this.volume_mwh),
          price_eur_per_mwh: Number(this.price_eur_per_mwh),
          id: uuid()
        });
        this.hideErrMsg();
        this.volume_mwh = 0;
        this.price_eur_per_mwh = 0;
      }
    }
  }

  hideErrMsg(): void {
    this.show_err_msg = false;
    this.inputs_err_msg_price = "";
    this.inputs_err_msg_volume = "";
    setTimeout(() => {
      this.show_err_msg = true;
    }, 200);
  }
}
</script>

<style scoped>
@media screen and (min-width: 350px) {
  .bid__container {
    display: grid;
    grid-template-areas:
      "qte_txt qte_input"
      "qte_err qte_err"
      "price_txt price_input"
      "price_err price_err"
      "btns btns";
    grid-template-columns: 1fr 1fr;
    grid-row: auto 20px auto 20px auto;
    max-width: 400px;
    margin: auto;
    gap: 0.2rem 1rem;
  }
  .bid__container_qte_txt,
  .bid__container_price_txt {
    justify-self: end;
    align-self: center;
  }
  .bid__container_qte_input,
  .bid__container_price_input {
    justify-self: start;
  }
}

@media screen and (max-width: 350px) {
  .bid__container {
    display: grid;
    grid-template-areas:
      "qte_txt"
      "qte_input"
      "qte_err"
      "price_txt"
      "price_input"
      "price_err"
      "btns";
    grid-template-columns: 1fr;
    grid-row: auto auto 20px auto auto 20px auto;
    margin: auto;
    gap: 0.6rem;
  }
  .bid__container_qte_input,
  .bid__container_price_input,
  .bid__container_qte_txt,
  .bid__container_price_txt {
    justify-self: center;
    align-self: center;
  }
}

.bid__container_qte_txt {
  grid-area: qte_txt;
}
.bid__container_qte_input {
  grid-area: qte_input;
}
.bid__container_qte_err {
  grid-area: qte_err;
}
.bid__container_price_txt {
  grid-area: price_txt;
}
.bid__container_price_input {
  grid-area: price_input;
}
.bid__container_price_err {
  grid-area: price_err;
}
.bid__container_actions {
  grid-area: btns;
}

.bid__container_qte_input > input,
.bid__container_price_input > input {
  text-align: center;
  font-size: 1.2rem;
  max-width: 100px;
}

.bid__container_qte_err,
.bid__container_price_err {
  font-size: 0.9rem;
  color: red;
  font-style: italic;
  text-align: center;
  height: 1rem;
}

.bid__container_actions button:first-of-type {
  margin-right: 1rem;
}
.bid__container_actions button:last-of-type {
  margin-left: 1rem;
}

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
</style>
