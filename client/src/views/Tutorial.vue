<template>
  <div class="tuto">
    <h2>{{ titles[tuto_step] }}</h2>
    <div class="tuto__nav">
        <Btn
          :background_color="'green'"
          @click="tuto_step -= 1"
          :disabled="tuto_step <= 0"
          :font_size="'0.9rem'"
          >Prec.</Btn
        >
        <Btn
          :background_color="'green'"
          @click="tuto_step += 1"
          :disabled="tuto_step >= titles.length - 1"
          :font_size="'0.9rem'"
          >Suivant</Btn
        >
    </div>

    <TutoEOD v-if="tuto_step === 0" class="tuto__content" />
    <TutoHome v-if="tuto_step === 1" class="tuto__content" />
    <TutoPowerPlants v-if="tuto_step === 2" class="tuto__content" />
    <TutoMarket v-if="tuto_step === 3" class="tuto__content" />
    <TutoOTC v-if="tuto_step === 4" class="tuto__content" />
    <TutoResults v-if="tuto_step === 5" class="tuto__content" />
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import Btn from "../components/base/Button.vue";
import TutoHome from "../components/TutoHome.vue";
import TutoEOD from "../components/TutoEOD.vue";
import TutoPowerPlants from "../components/TutoPowerPlants.vue";
import TutoMarket from "../components/TutoMarket.vue";
import TutoOTC from "../components/TutoOTC.vue";

@Component({
  components: {
    Btn,
    TutoHome,
    TutoEOD,
    TutoPowerPlants,
    TutoMarket,
    TutoOTC
  }
})
export default class Tutorial extends Vue {
  tuto_step = 0;
  titles = [
    "Equilibre offre-demande",
    "Phases de jeu",
    "Vos centrales",
    "Le marché",
    "Les échanges directs"
  ];
}
</script>

<style scoped>
.tuto h2 {
  display: inline-block;
  padding: 0 1rem;
  max-width: 300px;
  margin: 1rem auto 0rem;
}
.tuto__nav {
  margin: 0rem auto;
  max-width: 600px;
  box-sizing: border-box;
  margin: 0.8rem auto 1.2rem;
  padding: 0 1.5rem;

  display: flex;
  flex-direction: row;
  justify-content: center;
  align-items: center;
}
.tuto__nav button {
  width: 100px;
  margin: 0 1rem;
}
.tuto__content {
  margin: 0rem auto;
  padding: 0 1.5rem;
  max-width: 600px;
}

.tuto__content > :first-child {
  margin-top: 0;
}

.tuto__content >>> p {
  font-size: 1.2rem;
  text-align: start;
}
</style>
