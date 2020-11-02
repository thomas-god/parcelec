<template>
  <div>
    <MainDataForecast />
    <MainDataPlannings />
    <MainDataStocks />
  </div>
</template>

<script lang='ts'>
import { Component, Vue } from "vue-property-decorator";
import MainDataForecast from "./MainDataForecast.vue";
import MainDataPlannings from "./MainDataPlannings.vue";
import MainDataStocks from "./MainDataStocks.vue";

@Component({
  components: { MainDataForecast, MainDataPlannings, MainDataStocks }
})
export default class MainData extends Vue {}
</script>

<style>
</style>