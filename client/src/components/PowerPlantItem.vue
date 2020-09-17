<template>
  <div class="pp__grid">
    <div class="pp__logo">{{ logo }}</div>
    <div :style="pp__bare_style">
      <div :style="pp__bare__p_min" class="pp__bare__p_min">
        <p>{{ power_plant.p_min_mw }}</p>
      </div>
      <div class="pp__bare__p_max">
        <p>{{ power_plant.p_max_mw }}</p>
      </div>
    </div>
    <div class="pp__infos">
      Coût : <strong>{{ power_plant.price_eur_per_mwh }}</strong> €/MWh, Stock :
      <strong>{{ stock }}</strong> MWh
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
  @Prop() power_max_mw!: number;

  get stock(): string {
    return this.power_plant.stock_max_mwh === -1
      ? "∞"
      : String(this.power_plant.stock_max_mwh);
  }

  get logo(): string {
    let logo = "";
    if (this.power_plant.type === "nuc") logo = "☢️";
    if (this.power_plant.type === "therm") logo = "🔥";
    if (this.power_plant.type === "hydro") logo = "💧";
    if (this.power_plant.type === "ren") logo = "☀️";
    if (this.power_plant.type === "storage") logo = "🔋";
    return logo;
  }

  get pp__bare_style(): string {
    return `
      position: relative;
      grid-area: barre;
      border: 2px solid rgb(0, 195, 255);
      border-radius: 2px;
      width: ${(this.power_plant.p_max_mw / this.power_max_mw) * 100}%
    `;
  }

  get pp__bare__p_min(): string {
    return `
      width: ${(this.power_plant.p_min_mw / this.power_plant.p_max_mw) * 100}%;
    `;
  }
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
  align-self: center;
  text-align: center;
  font-size: 2rem;
}

.pp__barre {
  grid-area: barre;
  border: 1px solid rgb(136, 184, 199);
  border-radius: 2px;
}

.pp__bare__p_max {
  position: absolute;
  top: 0;
  right: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
}
.pp__bare__p_min {
  position: relative;
  height: 100%;
  text-align: end;
  border-right: 2px dashed black;
  background: repeating-linear-gradient(
    -45deg,
    #c8cad4,
    #c8cad4 10px,
    #a2a4aa 10px,
    #a2a4aa 20px
  );
  display: flex;
  flex-direction: column;
  justify-content: center;
}
.pp__bare__p_min p,
.pp__bare__p_max p {
  margin: 0;
  padding-right: 5px;
  font-weight: bold;
}

.pp__infos {
  grid-area: infos;
  align-self: center;
}
</style>