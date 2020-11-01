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
      prendre connaissance de vos centrales 
      {{
        conso_forecast.length > 0 ? " et des prévisions," : ","
      }}
      et quand vous serez prêt·e à
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

const user_module = namespace("user");
const session_module = namespace("session");
const portfolio_module = namespace("portfolio");

@Component({
  components: {
    Btn
  }
})
export default class MainWaitroom extends Vue {
  @State("api_url") api_url!: string;
  @user_module.Getter username!: string;
  @user_module.Getter user_id!: string;
  @user_module.State ready!: boolean;
  @user_module.Mutation SET_GAME_READY!: (game_ready: boolean) => void;
  @session_module.Getter session!: Session;
  @portfolio_module.Getter conso_forecast!: number[];

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
