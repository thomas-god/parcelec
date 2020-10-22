<template>
  <div>
    <p>
      Une partie de parcélec se compose d'une succession de <em>phases</em>. Au
      début de chaque phase vos équipes commerciales vous indiqueront quelle
      niveau de consommation vous aurez à couvrir pour la phase.
    </p>
    <p>
      Une fois une phase commencée vous avez un temps limité pour penser et
      exécuter toutes vos actions : achats/ventes sur le marché, allumer vos
      centrales, etc. Si vous pensez n'avoir plus rien à faire vous pouvez
      passer une phase et le jeu passera à la phase suivante si les autres
      joueurs sont pret·es à passer.
    </p>
    <p>
      Quand une phase se termine, vous recevrez vos resultats
      technico-financiers qui vous aideront à prendre de meilleures décisions
      dans les phases suivantes. Quand tous les joueurs sont pret·es, le jeu
      passera automatiquement à la phase suivante.
    </p>
  </div>
</template>

<script lang='ts'>
import { Component, Vue } from "vue-property-decorator";

@Component
export default class TutoHome extends Vue {
}
</script>

<style scoped>
</style>