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
    <div class="tuto__timers">
      <div v-if="time_before_clearing_ms > 0" class="tuto__timers_item">
        <span
          >Fermeture du marché dans
          <span class="tuto__timers_timer">{{
            toTimeString(time_before_clearing_ms)
          }}</span></span
        >
        <Btn @click="skipTime(bids_duration_s)">Passer</Btn>
      </div>
      <div v-else class="tuto__timers_closed">
        <span>Marché fermé</span>
      </div>

      <div v-if="time_before_planning_ms > 0" class="tuto__timers_item">
        <span>
          <span>
            Fin de la réception des plannings dans
            <span class="tuto__timers_timer">{{
              toTimeString(time_before_planning_ms)
            }}</span></span
          >
        </span>
        <Btn
          @click="skipTime(planning_duration_s)"
          v-if="time_before_clearing_ms < 0"
          >Passer</Btn
        >
      </div>
      <div v-else class="tuto__timers_closed">
        <span>Reception des plannings fermée</span>
      </div>
    </div>
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
import Btn from "./base/Button.vue";

@Component({ components: { Btn } })
export default class TutoHome extends Vue {
  /**
   * Timings
   */
  bids_duration_s = 120;
  planning_duration_s = 180;
  start = new Date();
  now: Date = new Date();
  dt_s = 0;
  interval = -1;
  mounted() {
    this.interval = window.setInterval(() => (this.now = new Date()), 1000);
  }
  beforeDestroy() {
    window.clearInterval(this.interval);
  }
  get time_before_clearing_ms(): number {
    return (
      this.start.valueOf() +
      this.bids_duration_s * 1000 -
      (this.now.valueOf() + this.dt_s * 1000)
    );
  }
  get time_before_planning_ms(): number {
    return (
      this.start.valueOf() +
      this.planning_duration_s * 1000 -
      (this.now.valueOf() + this.dt_s * 1000)
    );
  }

  toTimeString(dt_ms: number): string {
    if (dt_ms > 0) {
      const dt_s = dt_ms / 1000;
      const min = Math.floor(dt_s / 60);
      const sec = Math.floor(dt_s - min * 60);
      return `${String(min).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
    } else {
      return "";
    }
  }

  skipTime(dt_s: number): void {
    this.dt_s = dt_s;
    this.now = new Date();
  }
}
</script>

<style scoped>
.tuto__timers {
  max-width: 400px;
  margin: auto;
  padding: 1rem;
  border: 1px solid gray;
  border-radius: 1rem;
  font-size: 1rem;

  display: grid;
  grid-auto-columns: 1fr;
  grid-auto-rows: 2rem 2rem;
  gap: 0.6rem;
}

.tuto__timers_timer {
  display: inline-block;
  padding-left: 0.7ch;
  width: 55px;
  font-weight: bold;
}
.tuto__timers_closed {
  font-style: italic;
  color: red;
}
.tuto__timers_item,
.tuto__timers_closed {
  display: flex;
  flex-direction: row;
  justify-content: flex-start;
  align-items: center;
  text-align: start;
}
</style>