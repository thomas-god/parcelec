<template>
  <div>
    <p>
      L'équilibre offre-demande est un aspect fondamental de parcélec, et des
      systèmes électriques en général : il impose qu'à tout moment vous
      produisiez autant d'énergie que ce que vous (et vos clients) consommez. Si
      vous produisez trop ou pas assez d'énergie d'autres acteurs vont être
      appelés pour corriger votre écart et vous serez exposé·e à des pénalités
      financières !
    </p>
    <p>
      Pour vous aider à être à l'équilibre vous disposez d'un indicateur temps
      réel de votre position : par exemple vous êtes ici en déficit de 100 MWh.
      Allumez des centrales ou achetez de l'énergie sur le marché pour passer à
      0 !
    </p>
    <div class="tuto__bilan_eod">
      -100 MWh / 5600 €
    </div>
    <p>
      Une fois que vous êtes à l'équilibre il vous faudra encore produire au
      meilleure coût. Entre vos centrales et le marché il existera souvent
      plusieurs façons de vous mettre à l'équilibre, mais avec des coûts pouvant
      être très différents : à vous de trouver la combinaison optimale !
    </p>
    <p>
      Pour vous aider, le second chiffre, en €, correspond au coût actuel de
      votre position : essayez d'avoir le chiffre le plus bas tout en gardant
      une position à 0 !
    </p>
  </div>
</template>

<script lang='ts'>
import { Component, Vue } from "vue-property-decorator";

@Component
export default class TutoEOD extends Vue {}
</script>

<style scope>
.tuto__bilan_eod {
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