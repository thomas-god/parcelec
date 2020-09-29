<template>
  <div class="bilans__container">
    <div class="bilans__item">
      <span><em>Clients</em></span>
      <span> {{ fmt(conso) }} MWh </span>
      <span> {{ fmt(conso_eur) }} € </span>
    </div>
    <div class="bilans__item">
      <span><em>Production</em></span>
      <span> {{ fmt(prod_total_mwh) }} MWh </span>
      <span> {{ fmt(-1 * prod_eur) }} € </span>
    </div>
    <div class="bilans__item">
      <span><em>Ventes</em></span>
      <span> {{ fmt(sell.volume_mwh) }} MWh </span>
      <span> {{ fmt(sell_eur) }} € </span>
    </div>
    <div class="bilans__item">
      <span><em>Achats</em></span>
      <span> {{ fmt(buy.volume_mwh) }} MWh </span>
      <span> {{ fmt(-1 * buy_eur) }} € </span>
    </div>
    <div class="bilans__item">
      <span><em>Écarts</em></span>
      <span> {{ fmt(deficit_mwh) }} MWh </span>
      <span> {{ fmt(imbalance_costs_eur) }} € </span>
    </div>
    <div class="bilans__item">
      <span><em>Total</em></span>
      <span>-</span>
      <span> {{ fmt(money) }} € </span>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { BidsState, EnergyExchange } from "../store/bids";
import { PowerPlant } from "../store/portfolio";

const portfolioModule = namespace("portfolio");
const bidsModule = namespace("bids");
const resultsModule = namespace("results");
const sessionModule = namespace("session");

@Component
export default class Bilans extends Vue {
  @sessionModule.State results_available!: boolean;
  @portfolioModule.State power_plants!: PowerPlant[];
  @portfolioModule.State conso!: number;
  @bidsModule.State buy!: EnergyExchange;
  @bidsModule.State sell!: EnergyExchange;
  @resultsModule.State conso_eur!: number;
  @resultsModule.State prod_eur!: number;
  @resultsModule.State sell_eur!: number;
  @resultsModule.State buy_eur!: number;
  @resultsModule.State imbalance_costs_eur!: number;
  @resultsModule.State balance_eur!: number;

  get deficit_mwh() {
    return Number(
      this.buy.volume_mwh +
        this.prod_total_mwh -
        this.conso -
        this.sell.volume_mwh
    );
  }

  get money() {
    return Number(
      this.conso_eur +
        this.sell_eur -
        this.prod_eur -
        this.buy_eur +
        this.imbalance_costs_eur
    );
  }

  get prod_total_mwh(): number {
    let prod = 0;
    if (this.power_plants.length > 0) {
      prod = this.power_plants
        .map((pp) => Number(pp.planning_modif))
        .reduce((a, b) => a + b);
    }
    return prod;
  }

  fmt(nb: number): string {
    return nb.toLocaleString("fr-FR");
  }
}
</script>

<style scoped>
.bilans__container {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  justify-content: center;
  align-items: center;
}
.bilans__item {
  display: flex;
  flex-direction: column;
  margin: 1rem;
  flex-basis: auto;
  flex-grow: 0;
}
.bilans__item span {
  justify-self: start;
  text-align: start;
  white-space: nowrap;
}
.bilans__item span:not(:first-child) {
  padding-left: 1ch;
}
</style>
