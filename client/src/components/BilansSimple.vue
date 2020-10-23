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
import { OTC } from "../store/otc";
import { PowerPlant } from "../store/portfolio";

const portfolio_module = namespace("portfolio");
const bids_module = namespace("bids");
const otcs_module = namespace("otcs");
const user_module = namespace("user");

@Component
export default class PlanningBilansSimple extends Vue {
  @user_module.State username!: string;
  @portfolio_module.State power_plants!: PowerPlant[];
  @portfolio_module.State conso!: number;
  @bids_module.State buy!: EnergyExchange;
  @bids_module.State sell!: EnergyExchange;
  @otcs_module.Getter otcs_accepted!: OTC[];

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
  get otcs_total(): { volume_mwh: number; cost_eur: number } {
    const res = { volume_mwh: 0, cost_eur: 0 };
    this.otcs_accepted.forEach(otc => {
      if (
        (otc.user_from === this.username && otc.type === "sell") ||
        (otc.user_to === this.username && otc.type === "buy")
      ) {
        res.cost_eur += otc.volume_mwh * otc.price_eur_per_mwh;
        res.volume_mwh -= otc.volume_mwh;
      }
      if (
        (otc.user_from === this.username && otc.type === "buy") ||
        (otc.user_to === this.username && otc.type === "sell")
      ) {
        res.cost_eur -= otc.volume_mwh * otc.price_eur_per_mwh;
        res.volume_mwh += otc.volume_mwh;
      }
    });
    return res;
  }
  get cost_total(): number {
    return (
      this.cost_production +
      this.buy.price_eur_per_mwh * this.buy.volume_mwh -
      this.sell.price_eur_per_mwh * this.sell.volume_mwh  +
      this.otcs_total.cost_eur
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
      this.buy.volume_mwh + 
      this.otcs_total.volume_mwh
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
    let style = "";
    if (this.planning_delta_mwh !== 0) style += "color: red;";
    return style;
  }
}
</script>

<style scoped>
.bilans__delimited {
  padding: 0 0.5ch;
}
</style>
