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
    <PowerPlantsList
      :show_actions="false"
      :dummy="true"
      :show_title="false"
      class="tuto__pp"
    />
    <p>
      Une centrale possède 2 caractéristiques importantes : son coût (en €/MWh)
      et son stock (en MWh). Le prix d'une centrale inlfuera directement sur
      votre coût total si vous l'allumez, alors que son stock dimuera d'une
      phase à l'autre si vous l'utilisez. Une centrale ne pourra plus produire
      une fois sont stock épuisé, il faudra donc arbitrer en stock et coût de
      production !
    </p>
    <p>
      Il existe plusieurs types de centrales :
    </p>
    <ul>
      <li>
        ☢️ les centrales nucléaires ont un coût de production faible et des
        puissances maximales importantes mais ne peuvent fonctionner à faibles
        puissances,
      </li>
      <li>
        🔥 les centrales thermiques coûtent cher à produire mais sont très
        flexibles,
      </li>
      <li>
        💧 les centrales hydrauliques ne coutent rien à produire mais on un
        stock limité,
      </li>
      <li>
        🔋 les centrales de stockage peuvent être rechargées si leur stock est
        trop bas,
      </li>
      <li>
        ☀️ enfin les centrales renouvelables ne coûtent rien à produire mais
        vous ne pouvez pas choisir leur point de fonctionnement.
      </li>
    </ul>
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
      }
    ]);
  }
}
</script>

<style scoped>
.tuto__pp {
  margin-top: 1rem;
}
.tuto__pp :first-child {
  margin-bottom: 0rem !important;
}

ul {
  margin-top: 0;
  padding-left: 1rem;
}
</style>