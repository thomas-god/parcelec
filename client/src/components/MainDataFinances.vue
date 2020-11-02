<template>
  <div v-if="previous_plannings.length > 0">
    <h2 v-if="display_title">Données financières</h2>
    <MainDataFinancesGraph
      class="chart"
      :results_by_type="results"
      :n_phases="conso_forecast.length"
    />
  </div>
</template>

<script lang='ts'>
import { Component, Vue, Prop } from "vue-property-decorator";
import { namespace } from "vuex-class";
import { PhaseResults } from "../../../server/src/routes/types";
import { PhasePlanning } from "../store/results";
import MainDataFinancesGraph from "./MainDataFinancesGraph.vue";

const session_module = namespace("session");
const portfolio_module = namespace("portfolio");
const results_module = namespace("results");

@Component({ components: { MainDataFinancesGraph } })
export default class MainDataFinances extends Vue {
  @Prop({ default: true }) display_title!: boolean;
  @session_module.Getter phase_no!: number;
  @portfolio_module.Getter conso_forecast!: number[];
  @results_module.Getter previous_plannings!: PhasePlanning[];
  @results_module.State previous_results!: PhaseResults[];

  get results(): {
    type: string;
    name: string;
    color: string;
    values: number[];
  }[] {
    return [
      {
        type: "conso_eur",
        name: "Recettes clients",
        color: "#fce100",
        values: this.previous_results.map(res => res.conso_eur)
      },
      {
        type: "prod_eur",
        name: "Dépenses centrales",
        color: "#00bcf2",
        values: this.previous_results.map(res => -res.prod_eur)
      },
      {
        type: "sell_eur",
        name: "Ventes",
        color: "green",
        values: this.previous_results.map(res => res.sell_eur)
      },
      {
        type: "buy_eur",
        name: "Achats",
        color: "red",
        values: this.previous_results.map(res => -res.buy_eur)
      },
      {
        type: "imbalance_eur",
        name: "Ecarts",
        color: "#f5ad42",
        values: this.previous_results.map(res => res.imbalance_costs_eur)
      }
    ];
  }
}
</script>

<style scoped>
h2 {
  margin-top: 0;
  margin-bottom: 10px;
}
.chart {
  position: relative;
  max-width: 100%;
  height: 400px;
  margin-left: 0;
  padding: 5px 10px 5px 0;
}
@media screen and (min-width: 400px) {
  .chart {
    width: 80vw;
  }
}
@media screen and (max-width: 400px) {
  .chart {
    width: 95vw;
  }
}
</style>