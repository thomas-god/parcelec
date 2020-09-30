<template>
  <div>ENDGAME</div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { BidsState, EnergyExchange } from "../store/bids";
import { PowerPlant } from "../store/portfolio";

const portfolioModule = namespace("portfolio");
const bidsModule = namespace("bids");
const resultsModule = namespace("results");
const sessionModule = namespace("session");

@Component
export default class BilansEndGame extends Vue {}
</script>

<style scoped></style>
