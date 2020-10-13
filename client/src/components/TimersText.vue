<template>
  <div class="timers_text__container">
    <span v-if="time_before_clearing_string && !results_available" class="timers_text__item">
      <span
        v-if="time_before_clearing_string === 'Temps écoulé'"
        style="color: red;"
      >
        Enchères clôturées
      </span>
      <span v-else>
        Fin des enchères dans <strong>{{ time_before_clearing_string }}</strong>
      </span>
      <Btn :disabled="ready" v-show="can_bid" @click="setStatusReady">
        Passer
      </Btn>
    </span>
    <span v-if="time_before_planning_string && !results_available" class="timers_text__item">
      <span
        v-if="time_before_planning_string === 'Temps écoulé'"
        style="color: red;"
      >
        Réception des plannings fermée
      </span>
      <span v-else>
        Fermeture de la réception des plannings dans
        <strong>{{ time_before_planning_string }}</strong>
      </span>
      <Btn
        :disabled="ready"
        v-show="!can_bid && can_post_planning"
        @click="setStatusReady"
      >
        Passer
      </Btn>
    </span>
  </div>
</template>

<script lang='ts'>
import { Component, Vue } from "vue-property-decorator";
import { State, namespace } from "vuex-class";
import { Session } from "../store/session";
import Btn from "./base/Button.vue";

const session_module = namespace("session");
const user_module = namespace("user");

@Component({ components: { Btn } })
export default class TimersText extends Vue {
  // Session, user, and phase infos
  @State("api_url") api_url!: string;
  @session_module.Getter session!: Session;
  @session_module.Getter phase_infos!: Session["phase_infos"];
  @user_module.Getter user_id!: string;
  @user_module.Mutation SET_GAME_READY!: (game_ready: boolean) => void;

  // Abilities booleans
  @session_module.Getter can_bid!: boolean;
  @session_module.Getter can_post_planning!: boolean;
  @session_module.Getter clearing_available!: boolean;
  @session_module.Getter results_available!: boolean;

  async setStatusReady(): Promise<void> {
    const res = await fetch(
      `${this.api_url}/session/${this.session.id}/user/${this.user_id}/ready`,
      {
        method: "PUT"
      }
    );
    if (res.status === 201) this.SET_GAME_READY(true);
  }

  /**
   * Timings
   */
  now: Date = new Date();
  created() {
    setInterval(() => (this.now = new Date()), 1000);
  }

  get time_before_clearing_ms(): number {
    return this.phase_infos!.clearing_time!.valueOf() - this.now.valueOf();
  }
  get time_before_clearing_string(): string {
    if (this.can_bid && this.time_before_clearing_ms > 0)
      return toTimeString(this.time_before_clearing_ms);
    else return "Temps écoulé";
  }

  get time_before_planning_ms(): number {
    return this.phase_infos!.planning_time!.valueOf() - this.now.valueOf();
  }
  get time_before_planning_string(): string {
    if (this.can_post_planning && this.time_before_planning_ms > 0)
      return toTimeString(this.time_before_planning_ms);
    else return "Temps écoulé";
  }

  /**
   * Dynamic styles
   */
  get style_elapsed_time(): string {
    let style = "";
    style += `width: ${Math.min(
      100,
      Math.max(0, (1 - this.time_before_planning_ms / 1000 / 180) * 100)
    )}%;`;
    return style;
  }
  get style_legend_clearing(): string {
    let style = "";
    style += `left: calc(${(120 / 180) * 100}% - 0.5rem);`;
    return style;
  }
}

function toTimeString(dt: number): string {
  const dts = dt / 1000;
  const min = Math.floor(dts / 60);
  const sec = Math.floor(dts - min * 60);
  return `${String(min).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
}
</script>

<style scoped>
.timers_text__container {
  display: flex;
  flex-direction: column;
}
.timers_text__item {
  width: 100%;
  margin-bottom: 6px;
}
.timers_text__item strong {
  display: inline-block;
  text-align: start;
  width: 50px;
}

.timers_text__container button{
  margin-left: 1rem;
}
</style>