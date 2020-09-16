<template>
  <div class="pp__grid">
    <div class="pp__logo">{{ power_plant.type }}</div>
    <div class="pp__barre">
      {{ power_plant.p_min_mw }}, {{ power_plant.p_max_mw }}
    </div>
    <div class="pp__infos">
      {{ power_plant.price_eur_per_mwh }}, {{ power_plant.stock_max_mwh }}
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { PowerPlant } from "../store/portfolio";

@Component
export default class PowerPlantItem extends Vue {
  @Prop() power_plant!: PowerPlant;
}
</script>

<style scoped>
.pp__grid {
  display: grid;
  grid-template-areas:
    "logo barre"
    "vide infos";
  grid-template-columns: 50px 1fr;
  grid-template-rows: 50px 30px;
}

.pp__logo {
  grid-area: logo;
}

.pp__barre {
  grid-area: barre;
  border: 1px solid red;
}

.pp__infos {
  grid-area: infos;
}
</style>