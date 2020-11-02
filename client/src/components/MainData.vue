<template> 
<div>
    <MainDataForecast/>
    <MainDataPlannings/>
</div>
</template>

<script lang='ts'>
import { Component, Vue } from "vue-property-decorator";
import MainDataForecast from "./MainDataForecast.vue";
import MainDataPlannings from "./MainDataPlannings.vue";

@Component({ components: { MainDataForecast, MainDataPlannings } })
export default class MainData extends Vue {}
</script>

<style>
</style>