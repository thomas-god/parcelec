<template>
  <div class="home">
    <h1>Bienvenue sur Parcélec ! ⚡️</h1>
    <p>
      Parcélec est un jeu éducatif qui vous met dans la peau d'un gestionnaire
      de parc de production d'électricité. Votre but : fournir de l'énergie à
      vos clients en utilisant le plus efficacement possible les centrales en
      votre possession et en ayant recours si besoin au marché de l'électricité.
    </p>
    <h2>Déroulé d'une partie</h2>
    <p>
      Une partie se compose d'une succession de phases dans lesquelles vous
      serez confronté·e à différents niveaux de consommation.
    </p>
    <p>
      Au début de chaque phase vos équipes commerciales vous indiqueront quel
      niveau de consommation vous devrez couvrir (attention tout MWh non couvert
      vous expose à une pénalité financière !).
    </p>
    <p>
      Il vous faudra alors régler les points de consigne de vos centrales et
      acheter ou vendre de l'énergie à d'autres producteurs pour répondre au
      mieux à ce besoin. À la fin d'une phase vous recevrez un bilan de
      performances énergétiques et financières.
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
  margin: 0 1rem;
}
</style>
