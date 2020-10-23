<template>
  <div>
    <p>
      L'objectif de parcélec est simple : produire au
      <em>meilleur coût</em> l'énergie que vos clients <em>consomment</em>. Mais
      attention, si vous produisez trop ou pas assez vous vous exposez à des
      pénalités financières !
    </p>
    <p>
      Vous trouverez en bas de la page votre <em>écart actuel</em> (il vous
      manque actuellement 900 MWh) et votre
      <em>coût total de production</em> (0€ pour le moment). Allumez des
      centrales ou achetez de l'énergie sur le marché pour être à 0 tout en
      essayant d'avoir le coût de production le plus petit possible !
    </p>
    <p>
      Une partie se compose de plusieurs phases au court desquelles vos clients
      vont consommer différents volumes d'énergie. Le joeur qui l'emporte sera
      celui qui aura les plus gros gains à la fin de le partie !
    </p>
    <p>
      Les étapes suivantes du tutoriel vont vous montrer comment allumer et
      piloter vos centrales, poster des enchères sur le marché et acheter ou
      vendre de l'énergie directement aux autres joueurs.
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