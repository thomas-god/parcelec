<template>
  <div>
    <p>
      Vous êtes maintenant prêt·e à commencer une partie !
    </p>
    <div class="tuto__actions">
      <Btn font_size="1.2rem" @click="$router.push('/create')">
        Créer une partie
      </Btn>
      <Btn font_size="1.2rem" @click="$router.push('/join')">
        Rejoindre une partie
      </Btn>
    </div>
  </div>
</template>

<script lang='ts'>
import { Component, Vue } from "vue-property-decorator";
import Btn from "../components/base/Button.vue";

@Component({ components: { Btn } })
export default class TutoEnd extends Vue {}
</script>

<style scoped>
.tuto__actions button {
  margin: 0.5rem 1rem;
}
</style>