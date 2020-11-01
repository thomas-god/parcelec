<template>
  <div>
    <p>
      Prendre la meilleure décision peut s'avérer difficile, surtout si vous ne
      savez pas ce qu'il se passera dans les phases suivantes.
    </p>
    <p>
      Pour vous aider à prendre des décisions sur le long terme, vous disposez
      d'un onglet <em>Prévisions</em> 📊 qui vous indique les volumes de
      consommation susceptibles de se produire dans le futur.
    </p>
    <p>
      Suivant le scénario choisi les prévisions pourront être <em>parfaites</em> (vous savez exactement ce qui va se produire), <em>incertaines</em> (vous savez ce qui va se produire à une erreur prés) ou bien vous n'aurz <em>pas de prévisions</em>.
    </p>
    <ForecastGraph
      class="chart card"
      :data="conso_forecast"
      :line_title="'Consommation'"
      :y_title="'Consommation (MWh)'"
    />
  </div>
</template>

<script lang='ts'>
import { Component, Vue } from "vue-property-decorator";
import { namespace } from "vuex-class";
import ForecastGraph from "./ForecastGraph.vue";

const bids_module = namespace("bids");

@Component({ components: { ForecastGraph } })
export default class TutoForecast extends Vue {
  conso_forecast = [500, 900, 600, 1500, 2300, 1200];
}
</script>

<style scoped>
.card {
  padding: 1rem;
  border: 1px solid rgba(0, 0, 0, 0.493);
  border-radius: 3px;
  width: 100%;
  max-width: 500px;
  box-sizing: border-box;
  margin: auto;
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