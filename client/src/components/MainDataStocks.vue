<template>
  <div v-if="previous_plannings.length > 0">
    <h2>Evolutions de vos stocks</h2>
    <MainDataStocksGraph
      class="chart"
      :n_phases="conso_forecast.length"
      :stocks_by_type="stocks_by_type"
      :current_phase="phase_no"
    />
  </div>
</template>

<script lang='ts'>
import { Component, Vue } from "vue-property-decorator";
import { namespace } from "vuex-class";
import { PhaseResults } from "../../../server/src/routes/types";
import { PhasePlanning } from "../store/results";
import MainDataStocksGraph from "./MainDataStocksGraph.vue";

const session_module = namespace("session");
const portfolio_module = namespace("portfolio");
const results_module = namespace("results");

@Component({ components: { MainDataStocksGraph } })
export default class MainDataStocks extends Vue {
  @session_module.Getter phase_no!: number;
  @portfolio_module.Getter conso_forecast!: number[];
  @results_module.Getter previous_plannings!: PhasePlanning[];
  @results_module.State previous_results!: PhaseResults[];

  get stocks_by_type(): {
    type: string;
    name: string;
    color: string;
    values: number[];
  }[] {
    const stocks = [
      {
        type: "hydro",
        name: "Hydraulique",
        color: "#00bcf2",
        values: [] as number[]
      },
      {
        type: "storage",
        name: "Stockage",
        color: "#16c60c",
        values: [] as number[]
      }
    ];

    stocks.forEach(type => {
      type.values = [
        this.previous_plannings[0].planning
          .filter(item => item.type === type.type)
          .reduce((prev, cur) => prev + cur.stock_start_mwh, 0)
      ].concat(
        this.previous_plannings.map(phase => {
          return phase.planning
            .filter(item => item.type === type.type)
            .reduce((prev, cur) => prev + cur.stock_end_mwh, 0);
        })
      );
    });
    return stocks;
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
  height: 300px;
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