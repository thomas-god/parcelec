<template>
  <div>
    <div v-if="portfolio.length > 0">
      <h3>
        {{ options.name === "default" ? "Scénario par défault" : options.name }}
      </h3>
      <p>
        {{ options.description }}
      </p>
      <ul>
        <li>Difficulté : {{ difficulty }}</li>
        <li>Nombre de phases : {{ options.conso_forecast_mwh.length }}</li>
        <li>Multijoueur : {{ options.multi_game ? "Oui" : "Non" }}</li>
        <li>Prévisions : {{ forecast }}</li>
      </ul>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Session } from "../store/session";

const sessionModule = namespace("session");

@Component
export default class ScenarioIDCard extends Vue {
  @Prop({ default: {} }) options!: any;
  @Prop() portfolio!: any;

  get difficulty(): string {
    let str = "";
    if (this.options.difficulty === "easy") str = "Facile";
    if (this.options.difficulty === "medium") str = "Moyenne";
    if (this.options.difficulty === "hard") str = "Difficile";
    return str;
  }

  get forecast(): string {
    let str = "";
    if (this.options.conso_forecast_type === "none") str = "Pas de prévisions";
    if (this.options.conso_forecast_type === "perfect") str = "Parfaites";
    return str;
  }
}
</script>

<style scoped>
h3 {
  margin-top: 0;
  margin-bottom: 10px;
}

p {
  font-size: 1.1rem;
  text-align: start;
  margin-top: 0;
}

ul {
  text-align: start;
  padding-left: 1.5rem;
}
</style>
