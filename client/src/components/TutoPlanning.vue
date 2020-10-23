<template>
  <div>
    <p>
      L'ensemble des points de consigne de vos centrales constitue votre
      <em>programme d'appel</em>.
    </p>
    <p>
      Une fois qu'un programme d'appel vous satisfait vous devez l'envoyer en
      cliquant sur le bouton <em>Envoyer</em> en bas de la liste de vos
      centrales. Vous pouvez modifier autant de fois que vous le souhaitez votre
      programme d'appel avant la fin d'une phase.
    </p>
    <p>
      Attention, tant qu'un programme d'appel n'est pas envoyé il ne sera pas
      pris en compte ! Si vous avez des modifications non envoyées votre
      indicateur d'écart en bas de la page sera affiché en rouge.
    </p>
    <PowerPlantsList class="tuto__pplist" :dummy="true" />
  </div>
</template>

<script lang='ts'>
import { Component, Vue } from "vue-property-decorator";
import { namespace } from "vuex-class";
import { PowerPlant } from "../store/portfolio";
import PowerPlantsList from "./PowerPlantsList.vue";

const portfolio_module = namespace("portfolio");

@Component({ components: { PowerPlantsList } })
export default class TutoPlanning extends Vue {
  @portfolio_module.Mutation SET_POWER_PLANTS!: (
    power_plants: PowerPlant[]
  ) => void;
  @portfolio_module.State power_plants!: PowerPlant[];

  created() {
    // Set dummy portfolio for demonstration purpose
    this.SET_POWER_PLANTS([
      {
        id: "1",
        type: "nuc",
        p_min_mw: 500,
        p_max_mw: 1300,
        stock_max_mwh: -1,
        price_eur_per_mwh: 25,
        planning: 0,
        planning_modif: 0,
        stock_mwh: -1
      },
      {
        id: "2",
        type: "hydro",
        p_min_mw: 50,
        p_max_mw: 500,
        stock_max_mwh: 1000,
        price_eur_per_mwh: 0,
        planning: 0,
        planning_modif: 0,
        stock_mwh: 1000
      },
      {
        id: "3",
        type: "therm",
        p_min_mw: 150,
        p_max_mw: 600,
        stock_max_mwh: -1,
        price_eur_per_mwh: 65,
        planning: 0,
        planning_modif: 0,
        stock_mwh: -1
      },
      {
        id: "4",
        type: "ren",
        p_min_mw: 0,
        p_max_mw: 350,
        stock_max_mwh: -1,
        price_eur_per_mwh: 0,
        planning: 0,
        planning_modif: 0,
        stock_mwh: -1
      },
      {
        id: "5",
        type: "storage",
        p_min_mw: 100,
        p_max_mw: 500,
        stock_max_mwh: 1000,
        price_eur_per_mwh: 0,
        planning: 0,
        planning_modif: 0,
        stock_mwh: 500
      }
    ]);
  }
}
</script>

<style scoped>
.tuto__pplist {
  max-width: 700px;
  margin: auto;
  text-align: center;
}
@media screen and (min-width: 500px) {
  .tuto__pplist {
    width: 75%;
    padding: 1rem;
  }
}
@media screen and (max-width: 500px) {
  .tuto__pplist {
    width: 100%;
    padding: 0rem;
  }
}
</style>