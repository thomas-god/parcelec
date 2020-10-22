<template>
  <div>
    <p>
      L'équilibre offre-demande est un aspect fondamental de parcélec : il
      impose qu'à tout moment vous produisiez autant d'énergie que ce que vous
      (et vos clients) consommez. Si vous produisez trop ou pas assez d'énergie
      vous serez exposé·e à des pénalités financières ! Pour vous aider à être à
      l'équilibre vous disposez d'un indicateur temps réel de votre écart :
      actuellement vous êtes en déficit de 900 MWh, allumez des centrales ou
      achetez de l'énergie sur le marché pour passer à 0 !
    </p>
    <p>
      Une fois que vous êtes à l'équilibre il vous faudra encore produire au
      meilleure coût. Entre vos centrales et le marché il existera souvent
      plusieurs façons de couvrir votre consomamtion avec des coûts pouvant être
      très différents : à vous de trouver la combinaison optimale ! Pour vous
      aider, le second chiffre, en €, correspond au coût actuel de votre
      position : essayez d'avoir le chiffre le plus bas tout en gardant une
      position à 0 !
    </p>
  </div>
</template>

<script lang='ts'>
import { Component, Vue } from "vue-property-decorator";
import { namespace } from "vuex-class";

const portfolio_module = namespace("portfolio");

@Component
export default class TutoEOD extends Vue {
  @portfolio_module.Mutation SET_CONSO!: (conso: number) => void;

  created() {
    this.SET_CONSO(900);
  }
}
</script>

<style scoped>
</style>