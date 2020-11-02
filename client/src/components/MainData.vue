<template>
  <div>
    <MainDataForecast class="item"/>
    <MainDataFinances class="item"/>
    <MainDataPlannings class="item" />
    <MainDataStocks class="item"/>
  </div>
</template>

<script lang='ts'>
import { Component, Vue } from "vue-property-decorator";
import MainDataForecast from "./MainDataForecast.vue";
import MainDataFinances from "./MainDataFinances.vue";
import MainDataPlannings from "./MainDataPlannings.vue";
import MainDataStocks from "./MainDataStocks.vue";

@Component({
  components: {
    MainDataForecast,
    MainDataFinances,
    MainDataPlannings,
    MainDataStocks
  }
})
export default class MainData extends Vue {}
</script>

<style scoped>
.item {
  margin-bottom: 2rem;
}
</style>