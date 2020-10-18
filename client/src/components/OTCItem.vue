<template>
  <div class="otc_item__container">
    <div class="otc_item__users">
      <strong>{{ item.user_to }}</strong> ->
      <strong>{{ item.user_from }}</strong> ({{ item.status }})
    </div>
    <div class="otc_item__details">
      {{ OTCAction }} <strong>{{ item.volume_mwh }}</strong> MWh à
      <strong>{{ item.price_eur_per_mwh }}</strong> €/MWh
    </div>
    <div class="otc_item__details">
      Total <strong>{{priceTotal}} €</strong>
    </div>
  </div>
</template>

<script lang='ts'>
import { Vue, Component, Prop } from "vue-property-decorator";
import { OTC } from "../store/otc";

@Component
export default class OTCItem extends Vue {
  @Prop() item!: OTC;

  get OTCAction(): string {
    if (this.item.type === "buy") {
      return "Acheter";
    } else if (this.item.type === "sell") {
      return "Vendre";
    } else {
      return "";
    }
  }

  get priceTotal(): number {
    return (
      (this.item.type === "sell" ? 1 : -1) *
      this.item.volume_mwh *
      this.item.price_eur_per_mwh
    );
  }
}
</script>
<style scoped>
.otc_item__container {
  padding: 0 10px
}
.otc_item__users {
  text-align: start;
  padding-left: 1rem;
}
.otc_item__details {
  text-align: start;
}

</style>