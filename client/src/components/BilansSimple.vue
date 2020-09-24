<template>
  <div>{{ mwh_total_string }} MWh / {{ cost_total_string }} €</div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, namespace } from "vuex-class";
import { EnergyExchange } from "../store/bids";
import { PowerPlant } from "../store/portfolio";

const portfolioModule = namespace("portfolio");
const bidsModule = namespace("bids");

@Component
export default class PlanningBilansSimple extends Vue {
  @portfolioModule.State power_plants!: PowerPlant[];
  @portfolioModule.State conso!: number;
  @bidsModule.State buy!: EnergyExchange;
  @bidsModule.State sell!: EnergyExchange;

  get cost_production(): number {
    return this.power_plants
      .map(pp => {
        return Number(pp.planning_modif) * Number(pp.price_eur_per_mwh);
      })
      .reduce((a, b) => a + b, 0);
  }

  get cost_total(): number {
    return (
      this.cost_production +
      this.buy.price_eur_per_mwh -
      this.sell.price_eur_per_mwh
    );
  }

  get cost_total_string(): string {
    return this.cost_total.toLocaleString("fr-FR");
  }

  get mwh_total(): number {
    return (
      this.power_plants.reduce(
        (a, b) => a + Number(b.planning_modif),
        0 as number
      ) -
      this.conso -
      this.sell.volume_mwh +
      this.buy.volume_mwh
    );
  }

  get mwh_total_string(): string {
    return this.mwh_total.toLocaleString("fr-FR");
  }
}
</script>

<style scoped>
</style>
