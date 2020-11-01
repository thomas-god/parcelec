<template>
  <div class="tuto">
    <h2>{{ tuto_steps[tuto_step].title }}</h2>
    <div class="tuto__nav">
      <Btn
        :background_color="tuto_step > 0 ? 'green' : 'white'"
        @click="tuto_step -= 1"
        :disabled="tuto_step <= 0"
        :font_size="'0.9rem'"
        >Préc.</Btn
      >
      <Btn
        :background_color="tuto_step < tuto_steps.length - 1 ? 'green' : 'white'"
        @click="tuto_step += 1"
        :disabled="tuto_step >= tuto_steps.length - 1"
        :font_size="'0.9rem'"
        >Suivant</Btn
      >
    </div>
    <keep-alive>
      <component
        v-bind:is="tuto_steps[tuto_step].component"
        class="tuto__content"
      ></component>
    </keep-alive>

    <BilanSimple class="tuto__footer_bilans" />
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import Btn from "../components/base/Button.vue";
import BilanSimple from "../components/BilansSimple.vue";
import TutoEOD from "../components/TutoEOD.vue";
import TutoPowerPlants from "../components/TutoPowerPlants.vue";
import TutoPlanning from "../components/TutoPlanning.vue";
import TutoMarket from "../components/TutoMarket.vue";
import TutoOTC from "../components/TutoOTC.vue";
import TutoForecast from '../components/TutoForecast.vue'
import TutoEnd from "../components/TutoEnd.vue";

@Component({
  components: {
    Btn,
    BilanSimple,
    TutoEOD,
    TutoPowerPlants,
    TutoPlanning,
    TutoMarket,
    TutoOTC,
    TutoForecast,
    TutoEnd
  }
})
export default class Tutorial extends Vue {
  tuto_step = 0;
  tuto_steps = [
    { title: "But du jeu", component: TutoEOD },
    { title: "Vos centrales", component: TutoPowerPlants },
    { title: "Votre planning", component: TutoPlanning },
    { title: "Le marché", component: TutoMarket },
    { title: "Les échanges directs", component: TutoOTC },
    { title: "Les prévisions", component: TutoForecast },
    { title: "Commencer une partie !", component: TutoEnd }
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
  margin: 0rem auto 4rem;
  padding: 0 1.5rem;
  max-width: 600px;
}

.tuto__content > :first-child {
  margin-top: 0;
}

.tuto__content >>> p,
.tuto__content >>> li {
  font-size: 1.2rem;
  text-align: justify;
}

.tuto__footer_bilans {
  width: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: rgb(204, 218, 250);
  border-top: 1px solid black;
  position: fixed;
  bottom: 0;
  z-index: 10;
  font-size: 1.5rem;
  height: 2.2rem;
}
</style>
