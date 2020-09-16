<template>
  <div class="pp__list">
    <PowerPlantItem v-for="pp in power_plants" :power_plant="pp" :key="pp.id" />
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { PowerPlant } from "../store/portfolio";
import PowerPlantItem from "./PowerPlantItem.vue";

const userModule = namespace("user");
const sessionModule = namespace("session");
const portfolioModule = namespace("portfolio");

@Component({ components: { PowerPlantItem } })
export default class Main extends Vue {
  @portfolioModule.Getter power_plants!: PowerPlant[];
}
</script>

<style scoped>
.pp__list {
  max-width: 400px;
}
</style>