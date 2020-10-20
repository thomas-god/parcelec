<template>
  <div>
    <p>
      En plus du marché il existe une autre façon d'échanger de l'énergie avec
      les autres joueurs : les échanges directs ou contrats de grè à grè. De la
      même façon que pour le marché vous pouvez faire une ou plusieurs offres à
      un joueur en particulier (acheter ou vendre un volume d'énergie à un
      prix). Le joueur peut alors accepter ou refuser votre offre, sans qu'il
      n'y ait de règles comme pour le marché.
    </p>
    <p>
      Les contrats directs peuvent être passés jusqu'à la fin d'une phase. Les
      contrats sans réponse à la fin d'une phase sont considérés comme refusés.
    </p>
    <OTC :dummy="true" class="card" />
  </div>
</template>

<script lang='ts'>
import { Component, Vue } from "vue-property-decorator";
import { namespace } from "vuex-class";
import OTC from "./OTC.vue";

const session_module = namespace("session");

@Component({ components: { OTC } })
export default class TutoOTC extends Vue {
  @session_module.Mutation SET_USERS!: (users: { name: string }[]) => void;
  created() {
    this.SET_USERS([{ name: "Joueur 1" }]);
  }
}
</script>

<style scoped>
.card {
  padding: 1rem;
  border: 1px solid rgba(0, 0, 0, 0.493);
  border-radius: 3px;
  width: 100%;
  max-width: 500px;
  box-sizing: border-box;
  margin: auto;
}
</style>