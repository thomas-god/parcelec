<template>
  <div v-if="!(session.id && username)" class="app__full">
    <h1>Bienvenue sur Parcélec ! ⚡️</h1>
    <SessionSelect v-if="!session.id" />
    <UsernameSelect v-if="session.id && !username" />
  </div>
  <div v-else>
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
    <div v-if="session_status === 'running'" class="app__grid">
      <h1 class="app__grid_head">Phase de jeu en cours...</h1>
      <div class="app__grid_main">
        <h2 v-if="timeBeforeClearing">
          <span v-if="timeBeforeClearing === 'Temps écoulé'" style="color: red;"
            >Enchères clôturées</span
          >
          <span v-else>Fin des enchères dans {{ timeBeforeClearing }}</span>
        </h2>
        <h2 v-if="timeBeforePlanning">
          <span v-if="timeBeforePlanning === 'Temps écoulé'" style="color: red;"
            >Réception des plannings fermée</span
          >
          <span v-else
            >Fermeture de la réception des plannings dans
            {{ timeBeforePlanning }}</span
          >
        </h2>
        <div class="app__main" v-if="session.id && username">
          <PowerPlantsList class="app__main_item" />
          <Bilans class="app__main_item" />
          <BidsList class="app__main_item" />
        </div>
      </div>
      <!-- <Chatroom class="chatroom__grid" display_direction="column" /> -->
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Session } from "../store/session";
import SessionSelect from "./SessionSelect.vue";
import UsernameSelect from "./UsernameSelect.vue";
import Chatroom from "./Chatroom.vue";
import Messages from "./Messages.vue";
import Bid from "./SessionBid.vue";
import PowerPlantsList from "./PowerPlantsList.vue";
import BidsList from "./BidsList.vue";
import Bilans from "./Bilans.vue";

const userModule = namespace("user");
const sessionModule = namespace("session");

@Component({
  components: {
    SessionSelect,
    UsernameSelect,
    Chatroom,
    Bid,
    PowerPlantsList,
    BidsList,
    Bilans,
  },
})
export default class Main extends Vue {
  @userModule.Getter username!: string;
  @sessionModule.Getter session!: Session;
  @sessionModule.Getter session_status!: string;
  @sessionModule.Getter phase_infos!: Session["phase_infos"];

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
}

function toTimeString(dt: number): string {
  const dts = dt / 1000;
  const min = Math.floor(dts / 60);
  const sec = Math.floor(dts - min * 60);
  return `${String(min).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
}
</script>

<style scoped>
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
  margin-bottom: 1rem;
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
  flex-grow: 0;
  align-items: stretch;
  justify-content: center;
}

.app__main_item {
  border: 2px solid gray;
  border-radius: 2px;
  min-width: 400px;
  margin: 2rem;
}
</style>
