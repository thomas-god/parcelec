<template>
  <div v-if="previous_plannings.length > 0">
    <h2>Plannings des phase précédentes</h2>
    <MainDataPlanningsGraph
      class="chart"
      :conso="conso_forecast"
      :plannings_by_type="plannings_by_type"
      :current_phase="phase_no"
    />
  </div>
</template>

<script lang='ts'>
import { Component, Vue } from "vue-property-decorator";
import { namespace } from "vuex-class";
import { PhaseResults } from "../../../server/src/routes/types";
import { PhasePlanning } from "../store/results";
import MainDataPlanningsGraph from "./MainDataPlanningsGraph.vue";

const session_module = namespace("session");
const portfolio_module = namespace("portfolio");
const results_module = namespace("results");

@Component({ components: { MainDataPlanningsGraph } })
export default class MainDataPlannings extends Vue {
  @session_module.Getter phase_no!: number;
  @portfolio_module.Getter conso_forecast!: number[];
  @results_module.Getter previous_plannings!: PhasePlanning[];
  @results_module.State previous_results!: PhaseResults[];

  get plannings_by_type(): {
    type: string;
    name: string;
    color: string;
    values: number[];
  }[] {
    const plannings = [
      {
        type: "nuc",
        name: "Nucléaire",
        color: "#fce100",
        values: [] as number[]
      },
      {
        type: "hydro",
        name: "Hydraulique",
        color: "#00bcf2",
        values: [] as number[]
      },
      {
        type: "therm",
        name: "Thermique",
        color: "#f7630c",
        values: [] as number[]
      },
      {
        type: "storage",
        name: "Stockage",
        color: "#16c60c",
        values: [] as number[]
      }
    ];

    plannings.forEach(type => {
      type.values = this.previous_plannings.map(phase => {
        return phase.planning
          .filter(item => item.type === type.type)
          .reduce((prev, cur) => prev + cur.p_dispatch_mw, 0);
      });
    });
    plannings.push({
      type: "sell",
      name: "Ventes",
      color: "green",
      values: this.previous_results.map(res => -res.sell_mwh) as number[]
    });
    plannings.push({
      type: "buy",
      name: "Achats",
      color: "red",
      values: this.previous_results.map(res => res.buy_mwh) as number[]
    });
    return plannings;
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