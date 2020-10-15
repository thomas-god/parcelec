<template>
  <div>
    <span :style="style_bilans__mwh">{{ mwh_total_string }} MWh</span>
    <span class="bilans__delimited">/</span>
    <span>{{ cost_total_string }} €</span>
  </div>
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

  /**
   * Total costs in euros
   */
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

  /**
   * Total production in MWh
   */
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

  /**
   * Planning delta in MWh
   */
  get planning_delta_mwh(): number {
    return this.power_plants.reduce(
      (a, b) => a + (Number(b.planning_modif) - Number(b.planning)),
      0 as number
    );
  }

  /**
   * Dynamic styles
   */
  get style_bilans__mwh(): string {
    let style = '';
    if(this.planning_delta_mwh !== 0)
      style += 'color: red;';
    return style;
  }
}
</script>

<style scoped>
.bilans__delimited {
  padding: 0 0.5ch;
}
</style>
