<template>
  <div>
    <h2>Prévision de consommation</h2>
    <MainDataForecastGraph
      class="chart"
      :data="conso_forecast"
      :line_title="'Consommation'"
      :y_title="'Consommation (MWh)'"
    />
  </div>
</template>

<script lang='ts'>
import { Component, Vue } from "vue-property-decorator";
import { namespace } from "vuex-class";
import MainDataForecastGraph from "./MainDataForecastGraph.vue";

const portfolio_module = namespace("portfolio");

@Component({ components: { MainDataForecastGraph } })
export default class MainDataForecast extends Vue {
  @portfolio_module.Getter conso_forecast!: number[];
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