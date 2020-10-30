<template>
  <div class="container" v-if="phase_infos">
    <div class="phase_infos">
      <span>
        Phase
        <strong>{{
          `${phase_infos.phase_no + 1}/${phase_infos.nb_phases}`
        }}</strong>
        {{ `${results_available ? "(terminée)" : ""}` }}
      </span>
      |
      <span>
        Consommation :
        <strong>{{ conso.toLocaleString("fr-FR") }} MWh</strong>
      </span>
    </div>
    <MainInfosTimersText class="phase_infos_timers" />
    <Btn
      font_size="1.1rem"
      @click="setStatusReady"
      :disabled="ready"
      v-if="
        results_available &&
          phase_infos.phase_no + 1 < phase_infos.nb_phases
      "
    >
      Passer à la phase suivante
    </Btn>
    <Btn
      font_size="1.1rem"
      v-if="
        results_available && phase_infos.phase_no + 1 === phase_infos.nb_phases
      "
      @click="goToGameResults"
    >
      Résultats de la partie
    </Btn>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Session } from "../store/session";
import Btn from "./base/Button.vue";
import MainInfosTimersText from "./MainInfosTimersText.vue";

const userModule = namespace("user");
const sessionModule = namespace("session");
const portfolioModule = namespace("portfolio");

@Component({
  components: {
    Btn,
    MainInfosTimersText
  }
})
export default class MainInfos extends Vue {
  @State("api_url") api_url!: string;
  @sessionModule.Getter session_id!: string;
  @userModule.Getter user_id!: string;
  @userModule.State ready!: boolean;
  @userModule.Mutation SET_GAME_READY!: (game_ready: boolean) => void;
  @sessionModule.Getter phase_infos!: Session["phase_infos"];
  @portfolioModule.Getter conso!: number;
  @sessionModule.Getter results_available!: boolean;
  
  /**
   * Status ready and go to end game
   */
  async setStatusReady(): Promise<void> {
    const res = await fetch(
      `${this.api_url}/session/${this.session_id}/user/${this.user_id}/ready`,
      {
        method: "PUT"
      }
    );
    if (res.status === 201) this.SET_GAME_READY(true);
  }
  goToGameResults(): void {
    this.$router.push(
      `/session/${this.session_id}/user/${this.user_id}/results`
    );
  }
}
</script>

<style scoped>
.container {
  grid-area: head;
  margin-bottom: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: start;
}
.phase_infos {
  margin: 0 1rem;
  font-size: 1.15rem;
}
.phase_infos_timers {
  margin-top: 1rem;
}
</style>
