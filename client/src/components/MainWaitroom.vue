<template>
  <div class="app_waitroom">
    <h1>
      Bonjour {{ username }}, vous avez rejoint la partie
      <em>{{ session.name }}</em> !
    </h1>
    <p>
      Vous pouvez
      {{
        session.multi_game ? "discuter avec les autres joueurs connectés," : ""
      }}
      prendre connaissance de vos centrales, et quand vous serez prêt·e à
      démarrer la partie, cliquez sur le bouton
      <em>"Je suis prêt·e !"</em>
    </p>
    <Btn font_size="1.1rem" @click="setStatusReady" :disabled="ready">
      Je suis prêt·e !
    </Btn>
  </div>
</template>

<script lang="ts">
import { Component, Vue } from "vue-property-decorator";
import { State, namespace } from "vuex-class";
import { Session } from "../store/session";
import Btn from "./base/Button.vue";

const userModule = namespace("user");
const sessionModule = namespace("session");

@Component({
  components: {
    Btn
  }
})
export default class MainWaitroom extends Vue {
  @State("api_url") api_url!: string;
  @userModule.Getter username!: string;
  @userModule.Getter user_id!: string;
  @userModule.State ready!: boolean;
  @userModule.Mutation SET_GAME_READY!: (game_ready: boolean) => void;
  @sessionModule.Getter session!: Session;

  async setStatusReady(): Promise<void> {
    const res = await fetch(
      `${this.api_url}/session/${this.session.id}/user/${this.user_id}/ready`,
      {
        method: "PUT"
      }
    );
    if (res.status === 201) this.SET_GAME_READY(true);
  }
}
</script>

<style scoped>
h1 {
  margin-top: 0;
}
</style>
