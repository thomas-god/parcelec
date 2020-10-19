<template>
  <div
    class="otc_item__container"
    :class="item.status === 'rejected' ? 'otc_item__container_refused' : ''"
  >
    <div class="otc_item__user" v-if="type === 'send'">
      A <strong>{{ item.user_to }}</strong> ({{ otc_status }})
    </div>
    <div class="otc_item__user" v-if="type === 'received'">
      De <strong>{{ item.user_from }}</strong> ({{ otc_status }})
    </div>
    <div class="otc_item__details">
      {{ otc_action }} <strong>{{ item.volume_mwh }}</strong> MWh à
      <strong>{{ item.price_eur_per_mwh }}</strong> €/MWh
      <div>Total {{ price_total.toFixed(2) }} €</div>
    </div>
    <div
      class="otc_item__actions"
      v-if="
        item.status === 'pending' && type === 'received' && can_post_planning
      "
    >
      <Btn :background_color="'green'" @click="acceptOTC">OK</Btn>
      <Btn :background_color="'red'" @click="rejectOTC">X</Btn>
    </div>
  </div>
</template>

<script lang='ts'>
import { Vue, Component, Prop } from "vue-property-decorator";
import { State, namespace } from "vuex-class";
import { OTC } from "../store/otc";
import Btn from "./base/Button.vue";

const user_module = namespace("user");
const session_module = namespace("session");

@Component({ components: { Btn } })
export default class OTCItem extends Vue {
  @Prop() item!: OTC;
  @Prop() type!: "send" | "received";
  @State api_url!: string;
  @session_module.Getter session_id!: string;
  @user_module.Getter user_id!: string;
  @session_module.Getter can_post_planning!: boolean;

  get otc_action(): string {
    if (this.type === "send") {
      if (this.item.type === "buy") {
        return "Acheter";
      } else if (this.item.type === "sell") {
        return "Vendre";
      } else return "";
    } else if (this.type === "received") {
      if (this.item.type === "buy") {
        return "Vendre";
      } else if (this.item.type === "sell") {
        return "Acheter";
      } else return "";
    } else {
      return "";
    }
  }

  get otc_status(): string {
    switch (this.item.status) {
      case "pending":
        return "en cours";
      case "accepted":
        return "accepté";
      case "rejected":
        return "refusé";
      default:
        return "";
    }
  }

  get price_total(): number {
    return (
      (this.item.type === "sell" ? 1 : -1) *
      (this.type === "send" ? 1 : -1) *
      this.item.volume_mwh *
      this.item.price_eur_per_mwh
    );
  }

  async acceptOTC(): Promise<void> {
    const res = await fetch(
      `${this.api_url}/session/${this.session_id}/user/${this.user_id}/otc/${this.item.id}/accept`,
      {
        method: "PUT"
      }
    );
  }

  async rejectOTC(): Promise<void> {
    const res = await fetch(
      `${this.api_url}/session/${this.session_id}/user/${this.user_id}/otc/${this.item.id}/reject`,
      {
        method: "PUT"
      }
    );
  }
}
</script>
<style scoped>
.otc_item__container {
  padding: 0 10px;
  margin-bottom: 1rem;

  display: grid;
  grid-template-areas:
    "user actions"
    "details actions";
}
.otc_item__user {
  grid-area: user;
  text-align: start;
}
.otc_item__details {
  grid-area: details;
  text-align: start;
  padding-left: 1rem;
}
.otc_item__actions {
  align-self: center;
  grid-area: actions;
}
.otc_item__actions button:first-of-type {
  margin-right: 0.5rem;
}
.otc_item__actions button:last-of-type {
  margin-left: 0.5rem;
}

.otc_item__container_refused {
  color: rgb(160, 160, 160);
  font-style: italic;
}
</style>