<template>
  <div>
    <h2>Précédents plannings</h2>
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
import { PhasePlanning } from "../store/results";
import MainDataPlanningsGraph from "./MainDataPlanningsGraph.vue";

const session_module = namespace("session");
const portfolio_module = namespace("portfolio");
const results_module = namespace("results");

@Component({ components: { MainDataPlanningsGraph } })
export default class MainDataPlannings extends Vue {
  @portfolio_module.Getter conso_forecast!: number[];
  @results_module.Getter plannings!: PhasePlanning[];
  @session_module.Getter phase_no!: number;

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
      type.values = this.plannings.map(phase => {
        return phase.planning
          .filter(item => item.type === type.type)
          .reduce((prev, cur) => prev + cur.p_dispatch_mw, 0);
      });
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