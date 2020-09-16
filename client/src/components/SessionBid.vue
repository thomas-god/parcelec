<template>
  <div class="bid__box">
    <h2>Votre enchère</h2>
    <input type="number" :disabled="!can_bid" v-model="bid_value" />
    <button @click="submitBid" :disabled="!can_bid">
      {{ can_bid ? "Soumettre" : "Offre envoyée" }}
    </button>
    <span v-if="bid_value_err" style="color: red">{{ bid_value_err_msg }}</span>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Session } from "../store/session";

const sessionModule = namespace("session");
const userModule = namespace("user");

@Component
export default class Bid extends Vue {
  @sessionModule.Getter session!: Session;
  @sessionModule.Getter can_bid!: boolean;
  @sessionModule.Action updateBidAbility!: (bid_ability: boolean) => void;
  @userModule.Getter user_id!: string;
  @State("api_url") api_url!: string;

  private bid_value = 0;
  private bid_value_err = false;
  private bid_value_err_msg = "";

  async submitBid(): Promise<void> {
    console.log(this.bid_value);
    const res = await fetch(`${this.api_url}/session/${this.session.id}/bid`, {
      method: "PUT",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ user_id: this.user_id, bid: this.bid_value })
    });
    if (res.status === 201) {
      this.bid_value_err = false;
      this.bid_value_err_msg = "";
      this.updateBidAbility(false);
    } else {
      this.bid_value_err = true;
      this.bid_value_err_msg = await res.text();
    }
  }
}
</script>

<style scoped>
.bid__box {
  margin: 10px auto;
  padding: 2rem;
  max-width: 350px;

  display: flex;
  flex-direction: column;
  justify-items: center;
}

.bid__box input {
  font-size: 2rem;
  text-align: center;
}

.bid__box button {
  margin: 10px auto;
  flex-grow: 0;
  max-width: 200px;
  font-size: 1.3rem;
}
</style>
