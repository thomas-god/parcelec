<template>
  <div>
    <p>
      Votre levier principal pour être à l'équilibre est votre parc de
      production qui se compose de différentes centrales aux caractéristiques
      variées. Chaque centrale est représentée par une barre de taille
      proportionnelle à la puissance maximale qu'elle peut fournir. La partie
      grisée correspond à la puissance minimale d'une centrale en dessous de
      laquelle elle ne peut produire. Pour faire varier la puissance d'une
      centrale déplacez son curseur et observez comment votre position et votre
      coût varient.
    </p>
    <BilanSimple class="tuto__eod" />
    <p>
      Une centrale possède 2 caractéristiques importantes : son coût (en €/MWh)
      et son stock (en MWh). Le prix d'une centrale inlfuera directement sur
      votre coût total si vous l'allumez, alors que son stock dimuera d'une
      phase à l'autre si vous l'utilisez. Une centrale ne pourra plus produire
      une fois sont stock épuisé, il faudra donc arbitrer en stock et coût de
      production !
    </p>
    <p>
      Il existe plusieurs types de centrales : les centrales nucléaires ☢️ ont
      un coût de production faible et des puissances maximales importantes mais
      ne peuvent fonctionner à faibles puissances, les centrales thermiques 🔥
      coûtent cher à produire mais sont très flexibles, les centrales
      hydrauliques 💧 ne coutent rien à produire mais on un stock limité, les
      centrales de stockage 🔋 peuvent être rechargées si leur stock est trop
      bas, et enfin les centrales renouvelables ☀️ ne coûtent rien à produire
      mais vous ne pouvez pas choisir leur point de fonctionnement.
    </p>
    <p>
      L'ensemble des points de consigne de vos centrales constitue votre
      programme d'appel. Une fois qu'un programme d'appel vous satisfait vous
      devez l'envoyer en cliquant sur le bouton <em>Envoyer</em> en bas de la
      liste de vos centrales. Attention, tant qu'un programme d'appel n'est pas
      envoyé il ne sera pas pris en compte ! Votre position sera affichée en
      rouge quand vous avez des modifications non envoyées. Vous pouvez modifier
      autant de fois que vous le souhaitez votre programme d'appel avant la fin
      d'une phase.
    </p>
    <PowerPlantsList class="tuto__pplist" :dummy="true" />
  </div>
</template>

<script lang='ts'>
import { Component, Vue } from "vue-property-decorator";
import { namespace } from "vuex-class";
import { PowerPlant } from "../store/portfolio";
import PowerPlantsList from "./PowerPlantsList.vue";
import BilanSimple from "./BilansSimple.vue";

const portfolio_module = namespace("portfolio");

@Component({ components: { PowerPlantsList, BilanSimple } })
export default class TutoPowerPlants extends Vue {
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
        planning: 150,
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
  width: 75%;
  max-width: 700px;
  margin: auto;
  padding: 1rem;
  text-align: center;
}
.tuto__eod {
  width: 75%;
  height: 2rem;
  margin: auto;
  display: flex;
  flex-direction: row;
  justify-content: center;
  align-items: center;
  font-size: 1.3rem;

  border: 1px solid rgb(95, 95, 95);
  border-radius: 1rem;
  background-color: rgb(204, 218, 250);
}
</style>