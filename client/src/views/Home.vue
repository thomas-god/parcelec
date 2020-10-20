<template>
  <div class="home">
    <h1>Bienvenue sur Parcélec ! ⚡️</h1>
    <p>
      Parcélec est un jeu éducatif qui vous met dans la peau d'un gestionnaire
      de parc de production d'électricité. Votre but : fournir de l'énergie à
      vos clients en utilisant le plus efficacement possible les centrales en
      votre possession et en ayant recours si besoin au marché de l'électricité.
    </p>

    <div class="home__actions">
      <Btn font_size="1.2rem" @click="$router.push('/tutorial')">
        Tutoriel
      </Btn>
      <Btn font_size="1.2rem" @click="$router.push('/create')">
        Commencer
      </Btn>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import Btn from "../components/base/Button.vue";              

@Component({ components: { Btn } })
export default class Home extends Vue {}
</script>

<style scoped>
.home {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.home p {
  font-size: 1.2rem;
  text-align: start;
  margin: 0 0rem 1rem;
  word-break: break-word;
  hyphens: auto;
}

.home p {
  max-width: 600px;
}

@media screen and (min-width: 900px) {
  .home {
    padding: 0 1.5rem;
  }
}
@media screen and (max-width: 900px) {
  .home p {
    text-align: initial;
    padding: 0 1.5rem;
  }
}

.home__actions button {
  width: 140px;
  margin: 0.5rem 1rem;
}
</style>
