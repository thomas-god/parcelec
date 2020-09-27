<template>
  <div v-if="portfolio.length > 0">
    <h2>
      {{ options.name === "default" ? "Scénario par défault" : options.name }}
    </h2>
    <p>
      {{ options.description }}
    </p>
    <ul>
      <li>Difficulté : {{ difficulty }}</li>
      <li>Nombre de phases : {{ options.conso_forecast_mwh.length }}</li>
      <li>Multijoueur : {{ options.multi_game ? "Oui" : "Non" }}</li>
    </ul>
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
}
</script>

<style scoped>
h2 {
  margin-top: 0;
}

ul {
  padding: 0;
}
</style>
