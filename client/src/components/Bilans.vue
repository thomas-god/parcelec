<template>
  <div>
    <h2 style="margin-bottom: 0.8rem;">Votre bilan</h2>
    <div class="bilans__grid_item" style="margin-bottom: 1rem;">
      <span class="bilans__grid_item_value"
        >{{ `${deficit > 0 ? "+" : ""}${deficit}` }} MWh</span
      >
    </div>
    <div class="bilans__grid">
      <div
        v-for="item in bilans"
        :key="item.name"
        :style="`grid-area: ${item.name};`"
        class="bilans__grid_item"
      >
        <span class="bilans__grid_item_name">{{ item.name }}</span>
        <span class="bilans__grid_item_value">{{ item.value }} MWh</span>
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

@Component
export default class PlanningBilans extends Vue {
  @portfolioModule.State power_plants!: PowerPlant[];
  @portfolioModule.State conso!: number;
  @bidsModule.State buy!: EnergyExchange;
  @bidsModule.State sell!: EnergyExchange;

  get bilans() {
    return [
      { name: "Consommation", value: "-" + this.conso },
      { name: "Production", value: this.prod_total_mwh },
      { name: "Ventes", value: "-" + this.sell.volume_mwh },
      { name: "Achats", value: this.buy.volume_mwh },
    ];
  }

  get deficit() {
    return (
      this.buy.volume_mwh +
      this.prod_total_mwh -
      this.conso -
      this.sell.volume_mwh
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
.bilans__grid {
  display: grid;
  grid-template-areas:
    "Consommation Production"
    "Ventes Achats";
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
}

.bilans__grid_item {
  display: flex;
  flex-direction: column;
}
.bilans__grid_item_value {
  font-size: 1.7rem;
}
</style>
