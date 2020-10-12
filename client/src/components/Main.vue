<template>
  <div class="main">
    <!-- Open session -->
    <div
      v-if="session_status === 'open'"
      :class="session.multi_game ? 'app__waitroom' : 'app__waitroom_solo'"
    >
      <div class="app__waitroom__title">
        <h1>
          Bonjour {{ username }}, vous avez rejoint la partie
          <em>{{ session.name }}</em> !
        </h1>
        <p>
          Vous pouvez discuter avec les autres joueurs connectés, prendre
          connaissance de vos centrales, et quand vous serez prêt·e à démarrer
          la partie, cliquez sur le bouton
          <em>"Je suis prêt·e !"</em>
        </p>
        <Btn font_size="1.1rem" @click="setStatusReady" :disabled="ready">
          Je suis prêt·e !
        </Btn>
      </div>
      <Chatroom
        class="app__waitroom__chatroom"
        :display_ready="true"
        v-if="session.multi_game"
      />
      <PowerPlantsList class="app__waitroom__pplist" :dummy="true" />
    </div>

    <!-- Running session -->
    <div v-if="session_status !== 'open'" class="app__grid">
      <h1 class="app__grid_head" v-if="!results_available">
        Phase {{ `${phase_infos.phase_no + 1}/${phase_infos.nb_phases}` }}
      </h1>
      <h1 class="app__grid_head" v-if="results_available">
        Phase
        {{ `${phase_infos.phase_no + 1}/${phase_infos.nb_phases}` }} terminée
      </h1>
      <div class="app__grid_main">
        <h2>Consommation : {{ conso.toLocaleString("fr-FR") }} MWh</h2>
        <h3 v-if="timeBeforeClearing && !results_available">
          <span
            v-if="timeBeforeClearing === 'Temps écoulé'"
            style="color: red;"
          >
            Enchères clôturées
          </span>
          <span v-else
            >Fin des enchères dans
            <strong>{{ timeBeforeClearing }}</strong></span
          >
          <Btn :disabled="ready" v-show="can_bid" @click="setStatusReady"
            >Passer</Btn
          >
        </h3>
        <h3 v-if="timeBeforePlanning && !results_available">
          <span v-if="timeBeforePlanning === 'Temps écoulé'" style="color: red;"
            >Réception des plannings fermée</span
          >
          <span v-else
            >Fermeture de la réception des plannings dans
            <strong>{{ timeBeforePlanning }}</strong></span
          >
          <Btn
            :disabled="ready"
            v-show="!can_bid && can_post_planning"
            @click="setStatusReady"
            >Passer</Btn
          >
        </h3>
        <h3 v-if="results_available && session_nb_users > 1">
          Classement phase :
          <strong>{{ user_rankings.current }}/{{ session_nb_users }} </strong>
          (Total :
          <strong>{{ user_rankings.overall }}/{{ session_nb_users }} </strong>)
        </h3>
        <Bilans v-if="results_available" />
        <div class="app__main" v-if="session.id && username">
          <PowerPlantsList
            class="app__main_item"
            :show_actions="!results_available"
          />
          <BidsList class="app__main_item" />
        </div>
      </div>
    </div>
    <Btn
      class="ready__btn"
      font_size="1.2rem"
      @click="setStatusReady"
      :disabled="ready"
      v-if="
        results_available && phase_infos.phase_no + 1 < phase_infos.nb_phases
      "
    >
      Passer à la phase suivante
    </Btn>
    <Btn
      class="ready__btn"
      font_size="1.2rem"
      v-if="
        results_available && phase_infos.phase_no + 1 === phase_infos.nb_phases
      "
      @click="goToGameResults"
    >
      Résultats de la partie
    </Btn>
    <BilansSimple class="app__footer_bilans" v-if="session.id && username" />
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Session } from "../store/session";
import Chatroom from "./Chatroom.vue";
import Messages from "./Messages.vue";
import Bid from "./SessionBid.vue";
import PowerPlantsList from "./PowerPlantsList.vue";
import BidsList from "./BidsList.vue";
import BilansSimple from "./BilansSimple.vue";
import Bilans from "./Bilans.vue";
import Btn from "./base/Button.vue";

const userModule = namespace("user");
const sessionModule = namespace("session");
const portfolioModule = namespace("portfolio");
const resultsModule = namespace("results");

@Component({
  components: {
    Chatroom,
    Bid,
    PowerPlantsList,
    BidsList,
    BilansSimple,
    Bilans,
    Btn
  }
})
export default class Main extends Vue {
  @State("api_url") api_url!: string;
  @userModule.Getter username!: string;
  @userModule.Getter user_id!: string;
  @userModule.State ready!: boolean;
  @userModule.Mutation SET_GAME_READY!: (game_ready: boolean) => void;
  @sessionModule.Getter session!: Session;
  @sessionModule.Getter session_status!: string;
  @sessionModule.Getter phase_infos!: Session["phase_infos"];
  @sessionModule.Getter session_id!: string;
  @portfolioModule.Getter conso!: number;

  // Abilities booleans
  @sessionModule.Getter can_bid!: boolean;
  @sessionModule.Getter can_post_planning!: boolean;
  @sessionModule.Getter clearing_available!: boolean;
  @sessionModule.Getter results_available!: boolean;

  // Ranking
  @sessionModule.Getter session_nb_users!: number;
  @resultsModule.Getter user_rankings!: number;

  now: Date = new Date();
  created() {
    setInterval(() => (this.now = new Date()), 1000);
  }

  get timeBeforeClearing() {
    if (this.phase_infos?.clearing_time) {
      const dt = this.phase_infos.clearing_time.valueOf() - this.now.valueOf();
      if (dt > 0)
        return toTimeString(
          this.phase_infos.clearing_time.valueOf() - this.now.valueOf()
        );
      else return "Temps écoulé";
    } else {
      return null;
    }
  }

  get timeBeforePlanning() {
    if (this.phase_infos?.planning_time) {
      const dt = this.phase_infos.planning_time.valueOf() - this.now.valueOf();
      if (dt > 0)
        return toTimeString(
          this.phase_infos.planning_time.valueOf() - this.now.valueOf()
        );
      else return "Temps écoulé";
    } else {
      return null;
    }
  }

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

function toTimeString(dt: number): string {
  const dts = dt / 1000;
  const min = Math.floor(dts / 60);
  const sec = Math.floor(dts - min * 60);
  return `${String(min).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
}
</script>

<style scoped>
.main {
  height: calc(100%-36px);
  margin-bottom: 4.5rem;
}
/**
  Waitroom
*/

.app__waitroom_solo {
  display: grid;
  max-width: 1000px;
  grid-template-areas:
    "title"
    "pp";
  margin: auto;
  padding: 0 10px;
}

.app__waitroom {
  display: grid;
  max-width: 1000px;
  margin: auto;
  padding: 0 10px;
}
@media screen and (min-width: 750px) {
  .app__waitroom {
    grid-template-areas:
      "title title"
      "pp chat";
    grid-template-columns: auto auto;
    grid-template-rows: auto auto;
    gap: 1rem;
  }
  .app__waitroom__pplist,
  .app__waitroom__chatroom {
    padding: 1rem;
  }
}
@media screen and (max-width: 750px) {
  .app__waitroom {
    grid-template-areas:
      "title"
      "pp"
      "chat";
    grid-template-columns: auto;
    grid-template-rows: auto auto auto;
    gap: 1rem;
  }
  .app__waitroom__pplist,
  .app__waitroom__chatroom {
    padding: 1rem;
  }
}
.app__waitroom__title {
  grid-area: title;
}
.app__waitroom__title p {
  max-width: 650px;
  margin: auto;
  margin-bottom: 1rem;
  font-size: 1.1rem;
}
.app__waitroom__title button {
  margin-bottom: 1.3rem;
}
.app__waitroom__chatroom,
.app__waitroom__pplist {
  height: 100%;
  width: 100%;
  box-sizing: border-box;
  max-width: 650px;
  border: 1px solid black;
  justify-self: center;
}
.app__waitroom__chatroom {
  grid-area: chat;
}

.app__waitroom__pplist {
  grid-area: pp;
}

/**
  Game mode
*/
.app__grid {
  display: grid;
  width: 100%;
  height: 100%;
  grid-template-areas:
    "head head"
    "main  main";
  grid-template-rows: auto 1fr;
  grid-template-columns: 2fr 1fr;
}

.app__grid_head {
  grid-area: head;
  margin-bottom: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.app__grid_main {
  grid-area: main;
  display: flex;
  flex-direction: column;
}

.app__grid_main h2 {
  margin: 10px;
}
.app__grid_main h3 {
  margin: 0.2rem;
  font-weight: normal;
}
.app__grid_main button {
  margin-left: 1rem;
}

.app__main {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  align-items: stretch;
  justify-content: center;
}

.app__main_item {
  flex-grow: 1;
  max-width: 500px;
}

@media screen and (min-width: 400px) {
  .app__main_item {
    margin: 2rem;
    border-radius: 2px;
    border: 2px solid gray;
  }
  .app__footer_bilans {
    font-size: 2rem;
  }
}

@media screen and (max-width: 400px) {
  .app__main_item {
    margin: 1rem 3px;
    border: none;
    padding: 4px;
    position: relative;
  }

  .app__main_item::after {
    content: "";
    position: absolute;
    bottom: 0;
    left: 12.5%;
    width: 75%;
    height: 1px;
    border-bottom: 2px solid gray;
  }

  .app__footer_bilans {
    font-size: 1.7rem;
  }
}

.ready__btn {
  position: -webkit-sticky;
  position: sticky;
  bottom: 5rem;
  display: block;
  margin: auto;
  z-index: 12;
}

.app__footer_bilans {
  width: 100%;
  height: 4rem;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: rgb(204, 218, 250);
  border-top: 1px solid black;
  position: fixed;
  bottom: 0;
  z-index: 10;
}
</style>
