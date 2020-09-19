<template>
  <div>
    <h2 style="margin-bottom: 0.8rem;">Votre bilan</h2>
    <div class="bilans__grid_item" style="margin-bottom: 1rem;">
      <span class="bilans__grid_item_value">
        {{ `${deficit > 0 ? "+" : ""}${deficit.toLocaleString("fr-FR")}` }} MWh
        <span v-if="results_available">
          /
          {{
            `${deficit_money > 0 ? "+" : ""}${deficit_money.toLocaleString(
              "fr-FR"
            )}`
          }}
          €
        </span>
      </span>
    </div>
    <!-- Volumes -->
    <div class="bilans__grid_volume">
      <div
        v-for="item in bilans"
        :key="item.name"
        :style="`grid-area: ${item.zone};`"
        class="bilans__grid_item"
      >
        <span class="bilans__grid_item_name">{{ item.name }}</span>
        <span class="bilans__grid_item_value">{{ item.value }} MWh</span>
      </div>
    </div>

    <!-- Finances -->
    <div class="bilans__grid_prices" v-if="results_available">
      <div
        v-for="item in bilan_money"
        :key="item.name"
        :style="`grid-area: ${item.zone};`"
        class="bilans__grid_item"
      >
        <span class="bilans__grid_item_name">{{ item.name }}</span>
        <span class="bilans__grid_item_value">{{ item.value }} €</span>
      </div>
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
export default class PlanningBilans extends Vue {
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

  get bilans() {
    return [
      {
        name: "Consommation",
        value: "-" + this.conso.toLocaleString("fr-FR"),
        zone: "conso",
      },
      {
        name: "Production",
        value: this.prod_total_mwh.toLocaleString("fr-FR"),
        zone: "prod",
      },
      {
        name: "Ventes",
        value: "-" + this.sell.volume_mwh.toLocaleString("fr-FR"),
        zone: "sell",
      },
      {
        name: "Achats",
        value: this.buy.volume_mwh.toLocaleString("fr-FR"),
        zone: "buy",
      },
    ];
  }

  get bilan_money() {
    return [
      {
        name: "Consommation",
        value: this.conso_eur.toLocaleString("fr-FR"),
        zone: "conso",
      },
      {
        name: "Production",
        value: "-" + this.prod_eur.toLocaleString("fr-FR"),
        zone: "prod",
      },
      {
        name: "Ventes",
        value: this.sell_eur.toLocaleString("fr-FR"),
        zone: "sell",
      },
      {
        name: "Achats",
        value: "-" + this.buy_eur.toLocaleString("fr-FR"),
        zone: "buy",
      },
      {
        name: "Règlement des écarts",
        value: this.imbalance_costs_eur.toLocaleString("fr-FR"),
        zone: "imbalance",
      },
    ];
  }

  get deficit() {
    return Number(
      this.buy.volume_mwh +
        this.prod_total_mwh -
        this.conso -
        this.sell.volume_mwh
    );
  }

  get deficit_money() {
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
}
</script>

<style scoped>
.bilans__grid_volume {
  display: grid;
  grid-template-areas:
    "conso prod"
    "sell buy";
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
  margin-bottom: 2rem;
}

.bilans__grid_prices {
  display: grid;
  grid-template-areas:
    "conso prod"
    "sell buy"
    "imbalance imbalance";
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr 1fr;
}

.bilans__grid_item {
  display: flex;
  flex-direction: column;
}
.bilans__grid_item_value {
  font-size: 1.5rem;
}
</style>
