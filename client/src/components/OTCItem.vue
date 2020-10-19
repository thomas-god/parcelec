<template>
  <div class="otc_item__container">
    <div class="otc_item__users" v-if="type === 'send'">
      A <strong>{{ item.user_to }}</strong> ({{ otc_status }})
    </div>
    <div class="otc_item__users" v-if="type === 'received'">
      De <strong>{{ item.user_from }}</strong> ({{ otc_status }})
    </div>
    <div class="otc_item__details">
      {{ otc_action }} <strong>{{ item.volume_mwh }}</strong> MWh à
      <strong>{{ item.price_eur_per_mwh }}</strong> €/MWh (total
      <strong>{{ price_total.toFixed(2) }} €</strong>)
    </div>
  </div>
</template>

<script lang='ts'>
import { Vue, Component, Prop } from "vue-property-decorator";
import { OTC } from "../store/otc";

@Component
export default class OTCItem extends Vue {
  @Prop() item!: OTC;
  @Prop() type!: "send" | "received";

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
}
</script>
<style scoped>
.otc_item__container {
  padding: 0 10px;
}
.otc_item__users {
  text-align: start;
  padding-left: 1rem;
}
.otc_item__details {
  text-align: start;
}
</style>