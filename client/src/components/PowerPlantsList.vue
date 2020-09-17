<template>
  <div class="pp__list">
    <h2>Votre parc de production</h2>
    <PowerPlantItem
      class="pp__list_item"
      v-for="pp in pp_sorted"
      :key="`pp-list-${pp.id}`"
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
export default class PowerPlantsList extends Vue {
  @portfolioModule.Getter power_plants!: PowerPlant[];

  get pp_sorted(): PowerPlant[] {
    return this.power_plants
      .map(pp => pp)
      .sort((a, b) => b.p_max_mw - a.p_max_mw);
  }

  get power_plants_max_power_mw(): number {
    return this.pp_sorted[0].p_max_mw;
  }
}
</script>

<style scoped>
.pp__list_item {
  margin: 1rem;
}
</style>