<template>
  <div>
    <div class="new_otc__inputs__container">
      <div class="new_otc__input_item">
        <label for="new__otc_user_to">Joueur</label>
        <select id="new__otc_user_to" v-model="user_to">
          <option
            v-for="user in otherUsers"
            :value="user.name"
            :key="user.name"
            >{{ user.name }}</option
          >
        </select>
        <span class="err_msg">{{
          show_err_msg ? inputs_err_msg_user_to : ""
        }}</span>
      </div>

      <div class="new_otc__input_item">
        <label for="new__otc_type">Type</label>
        <select name="Achat ou vente" id="new__otc_user_to" v-model="type">
          <option value="buy">Acheter</option>
          <option value="sell">Vendre</option>
        </select>
        <span class="err_msg">{{
          show_err_msg ? inputs_err_msg_type : ""
        }}</span>
      </div>

      <div class="new_otc__input_item">
        <label for="new__otc_volume_mwh">Volume (MWh)</label>
        <input
          type="number"
          id="new__otc_volume_mwh"
          min="0"
          step="1"
          v-model="volume_mwh"
        />
        <span class="err_msg">{{
          show_err_msg ? inputs_err_msg_volume : ""
        }}</span>
      </div>

      <div class="new_otc__input_item">
        <label for="new__otc_price_eur_per_mwh">Prix (€/MWh) </label>
        <input
          type="number"
          id="new__otc_price_eur_per_mwh"
          step="0.01"
          v-model="price_eur_per_mwh"
        />
        <span class="err_msg">{{
          show_err_msg ? inputs_err_msg_price : ""
        }}</span>
      </div>
    </div>
    <Btn @click="postOTC"> Envoyer </Btn>
  </div>
</template>

<script lang='ts'>
import { Component, Vue, Watch, Prop } from "vue-property-decorator";
import { State, namespace } from "vuex-class";
import { v4 as uuid } from "uuid";
import { OTC } from "../store/otc";
import { User } from "../store/session";
import Btn from "./base/Button.vue";

const user_module = namespace("user");
const otcs_module = namespace("otcs");
const session_module = namespace("session");

@Component({ components: { Btn } })
export default class NewOTC extends Vue {
  @Prop({ default: false }) dummy!: boolean;
  @session_module.State users!: User[];
  @user_module.Getter username!: string;
  @otcs_module.State otcs!: OTC[];

  get otherUsers(): User[] {
    return this.users.filter(u => u.name !== this.username);
  }

  /**
   * Inputs validation
   */
  user_to = "";
  type = "";
  volume_mwh = "";
  price_eur_per_mwh = "";

  inputs_err_msg_user_to = "";
  inputs_err_msg_type = "";
  inputs_err_msg_volume = "";
  inputs_err_msg_price = "";
  show_err_msg = true;

  validateInputs(): boolean {
    return [
      this.validateVolume(),
      this.validatePrice(),
      this.validateUserTo(),
      this.validateType()
    ].every(val => val);
  }

  @Watch("user_to")
  validateUserTo(): boolean {
    const flag = this.otherUsers.map(u => u.name).includes(this.user_to);
    this.inputs_err_msg_user_to = flag ? "" : "Choissez un joueur";
    return flag;
  }

  @Watch("type")
  validateType(): boolean {
    const flag = ["sell", "buy"].includes(this.type);
    this.inputs_err_msg_type = flag ? "" : "Choissez une action";
    return flag;
  }

  @Watch("volume_mwh")
  validateVolume(): boolean {
    let flag = true;
    if (this.show_err_msg) {
      if (isNaN(Number(this.volume_mwh)) || this.volume_mwh === "") {
        flag = false;
        this.inputs_err_msg_volume = "Le volume doit être un nombre";
      } else if (Number(this.volume_mwh) <= 0) {
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

  hideErrMsg(): void {
    this.show_err_msg = false;
    setTimeout(() => {
      this.inputs_err_msg_user_to = "";
      this.inputs_err_msg_type = "";
      this.inputs_err_msg_price = "";
      this.inputs_err_msg_volume = "";
      this.show_err_msg = true;
    }, 200);
  }

  /**
   * POST OTC
   */
  @State api_url!: string;
  @session_module.Getter session_id!: string;
  @user_module.Getter user_id!: string;
  @otcs_module.Mutation PUSH_OTC!: (otc: OTC) => void;
  async postOTC() {
    if (!this.dummy && this.validateInputs()) {
      const res = await fetch(
        `${this.api_url}/session/${this.session_id}/user/${this.user_id}/otc`,
        {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            user_to: this.user_to,
            type: this.type as "sell" | "buy",
            volume_mwh: Number(this.volume_mwh),
            price_eur_per_mwh: Number(this.price_eur_per_mwh),
          })
        }
      );
      if (res.status === 201) {
        const otc_id = (await res.json()).otc_id;
        this.PUSH_OTC({
          id: otc_id,
          user_from: this.username,
          user_to: this.user_to,
          type: this.type as "sell" | "buy",
          volume_mwh: Number(this.volume_mwh),
          price_eur_per_mwh: Number(this.price_eur_per_mwh),
          status: "pending"
        });
        this.hideErrMsg();
        this.user_to = "";
        this.type = "";
        this.volume_mwh = "";
        this.price_eur_per_mwh = "";
      } else {
        console.log(await res.text());
      }
    } else if (this.validateInputs()) {
      this.PUSH_OTC({
        id: uuid(),
        user_from: this.username,
        user_to: this.user_to,
        type: this.type as "sell" | "buy",
        volume_mwh: Number(this.volume_mwh),
        price_eur_per_mwh: Number(this.price_eur_per_mwh),
        status: "pending"
      });
      this.hideErrMsg();
      this.user_to = "";
      this.type = "";
      this.volume_mwh = "";
      this.price_eur_per_mwh = "";
    }
  }
}
</script>

<style scoped>
.new_otc__inputs__container {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-bottom: 1rem;
}

.new_otc__input_item {
  display: grid;
  grid-template-areas:
    "label input"
    "err err";
  grid-template-columns: 130px 150px;
  grid-template-rows: 1.7rem 1.4rem;
  margin-bottom: 0rem;
  column-gap: 1rem;
  align-items: center;
}
.new_otc__input_item label {
  grid-area: label;
}
.new_otc__input_item input {
  grid-area: input;
}
.new_otc__input_item .err_msg {
  grid-area: err;
  text-align: end;
  font-style: italic;
  font-size: 0.9rem;
  color: red;
}

input,
select {
  box-sizing: border-box;
  width: 150px;
  height: 1.7rem;
}
label {
  display: inline-block;
  margin-right: 1rem;
  width: 130px;
  text-align: end;
}

select,
select > option,
input[type="number"] {
  font-family: Montserrat;
  font-size: 1rem;
  text-align: center;
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