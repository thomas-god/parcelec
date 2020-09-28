<template>
  <div class="main">
    <!-- Open session -->
    <div v-if="session_status === 'open'" class="app__full">
      <h1>
        Bonjour {{ username }}, vous avez rejoint la partie
        <em>{{ session.name }}</em> !
      </h1>
      <h3>
        Vous pouvez discuter avec les autres joueurs connectés, et quand vous
        serez prêt·e à démarrer la partie, cliquez sur le bouton
        <em>"Je suis prêt·e!"</em>
      </h3>
      <Chatroom class="chatroom__full" :display_ready="true" />
    </div>

    <!-- Running session -->
    <div v-if="session_status === 'running'" class="app__grid">
      <h1 class="app__grid_head" v-if="!results_available">
        Phase de jeu en cours...
      </h1>
      <h1 class="app__grid_head" v-if="results_available">
        Phase de jeu terminée
      </h1>

      <div class="app__grid_main">
        <h2 v-if="timeBeforeClearing && !results_available">
          <span v-if="timeBeforeClearing === 'Temps écoulé'" style="color: red;"
            >Enchères clôturées</span
          >
          <span v-else>Fin des enchères dans {{ timeBeforeClearing }}</span>
        </h2>
        <h2 v-if="timeBeforePlanning && !results_available">
          <span v-if="timeBeforePlanning === 'Temps écoulé'" style="color: red;"
            >Réception des plannings fermée</span
          >
          <span v-else
            >Fermeture de la réception des plannings dans
            {{ timeBeforePlanning }}</span
          >
        </h2>
        <div class="app__main" v-if="session.id && username">
          <PowerPlantsList
            class="app__main_item"
            :show_actions="!results_available"
          />
          <BidsList class="app__main_item" />
        </div>
      </div>
    </div>
    <button
      class="ready__btn"
      @click="setStatusReady"
      :disable="!ready"
      v-if="results_available"
    >
      Passer à la phase suivante
    </button>
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

const userModule = namespace("user");
const sessionModule = namespace("session");

@Component({
  components: {
    Chatroom,
    Bid,
    PowerPlantsList,
    BidsList,
    BilansSimple,
  },
})
export default class Main extends Vue {
  @userModule.Getter username!: string;
  @userModule.Getter user_id!: string;
  @userModule.State ready!: boolean;
  @sessionModule.Getter session!: Session;
  @sessionModule.Getter session_status!: string;
  @sessionModule.Getter phase_infos!: Session["phase_infos"];
  @sessionModule.Getter session_id!: string;
  @State("api_url") api_url!: string;
  @userModule.Mutation SET_GAME_READY!: (game_ready: boolean) => void;

  // Abilities booleans
  @sessionModule.Getter can_bid!: boolean;
  @sessionModule.Getter can_post_planning!: boolean;
  @sessionModule.Getter clearing_available!: boolean;
  @sessionModule.Getter results_available!: boolean;

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
        method: "PUT",
      }
    );
    if (res.status === 201) this.SET_GAME_READY(true);
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

.app__full h3 {
  max-width: 650px;
  margin: auto;
  margin-bottom: 2rem;
}

.chatroom__full {
  width: 85vw;
  margin: auto;
}

.chatroom__grid {
  grid-area: message;
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
  border: none;
  font-size: 1.2rem;
  padding: 0.3rem 1.2rem 0.3rem 1.2rem;
  border-radius: 2em;
  color: white;
  background-color: #4eb5f1;
  text-align: center;
  transition: all 0.2s;
  position: -webkit-sticky;
  position: sticky;
  bottom: 5rem;
  display: block;
  margin: auto;
  z-index: 12;
}
.ready__btn:hover {
  background-color: #4095c6;
}
.ready__btn:active {
  font-size: 1.1rem;
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
