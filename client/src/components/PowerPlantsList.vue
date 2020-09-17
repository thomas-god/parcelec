<template>
  <div class="pp__list">
    <h2>Votre parc de production</h2>
    <PowerPlantItem
      class="pp__list_item"
      v-for="pp in power_plants"
      :key="pp.id"
      :power_plant="pp"
      :power_max_mw="power_plants_max_power_mw"
    />
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

  get power_plants_max_power_mw(): number {
    return this.power_plants.sort((a, b) => b.p_max_mw - a.p_max_mw)[0]
      .p_max_mw;
  }
}
</script>

<style scoped>
.pp__list {
  max-width: 400px;
  border: 2px solid gray;
  border-radius: 2px;
}

.pp__list_item {
  margin: 1rem;
}
</style>