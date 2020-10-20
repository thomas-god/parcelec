<template>
  <div class="tuto">
    <h2 class="tuto_title">{{titles[tuto_step]}}</h2>

    <div class="tuto__nav">
      <Btn
        :background_color="'green'"
        @click="tuto_step -= 1"
        :disabled="tuto_step === 0"
        >⬅️</Btn
      >
      {{ tuto_step }}/7
      <Btn
        :background_color="'green'"
        @click="tuto_step += 1"
        :disabled="tuto_step === 5"
        >➡️</Btn
      >
    </div>

    <TutoHome v-show="tuto_step === 0" class="tuto__content" />
    <TutoEOD v-show="tuto_step === 1" class="tuto__content" />
    <TutoPowerPlants v-show="tuto_step === 2" class="tuto__content" />
    <TutoMarket v-show="tuto_step === 3" class="tuto__content" />
    <TutoOTC v-show="tuto_step === 4" class="tuto__content" />
    <TutoResults v-show="tuto_step === 5" class="tuto__content" />
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import Btn from "../components/base/Button.vue";
import TutoHome from "../components/Tuto0Home.vue";
import TutoEOD from "../components/Tuto1EOD.vue";
import TutoPowerPlants from "../components/Tuto2PowerPlants.vue";
import TutoMarket from "../components/Tuto3Market.vue";
import TutoOTC from "../components/Tuto4OTC.vue";
import TutoResults from "../components/Tuto5Results.vue";

@Component({
  components: {
    Btn,
    TutoHome,
    TutoEOD,
    TutoPowerPlants,
    TutoMarket,
    TutoOTC,
    TutoResults
  }
})
export default class Home extends Vue {
  tuto_step = 0;
  titles = [
    'Tutoriel',
    'Equilibre offre-demande',
    'Vos centrales',
    'Le marché',
    'Les échanges directs',
    'Résultats'
  ]
}
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

.card {
  padding: 1rem;
  border: 1px solid rgba(0, 0, 0, 0.493);
  border-radius: 3px;
  width: 100%;
  max-width: 500px;
  box-sizing: border-box;
}

@media screen and (min-width: 900px) {
  .home {
    padding: 0 1.5rem;
  }
  .grid__left {
    display: grid;
    grid-template-columns: 1fr 1fr;
    align-items: center;
    gap: 2rem;
  }
  .grid__left .card {
    justify-self: end;
  }

  .grid__right {
    display: grid;
    grid-template-columns: 1fr 1fr;
    align-items: center;
    gap: 2rem;
  }
  .grid__right .card {
    justify-self: start;
  }
  .grid__right p {
    text-align: end;
  }
}
@media screen and (max-width: 900px) {
  .grid__left {
    display: grid;
    grid-template-rows: auto auto;
    justify-items: center;
    align-items: flex-start;
  }
  .grid__left p,
  .grid__right p,
  .home p {
    text-align: initial;
    padding: 0 1.5rem;
  }

  .grid__right {
    display: grid;
    grid-template-rows: auto auto;
    justify-items: center;
  }
  .grid__right > .card {
    grid-row-start: 1;
  }
  .grid__left .card,
  .grid__right .card {
    margin-bottom: 1rem;
  }
}

@media screen and (max-width: 520px) {
  .card {
    border: none;
    position: relative;
  }
  .card::before,
  .card::after {
    content: "";
    position: absolute;
    left: 12.5%;
    width: 75%;
    height: 1px;
    border-bottom: 2px solid gray;
  }
  .card::before {
    top: 0;
  }
  .card::after {
    bottom: 0;
  }
}

.home__create_game {
  margin: 1rem 1rem 3rem 1rem;
}
</style>
