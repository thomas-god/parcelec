<template>
  <div class="main" id="main">
    <!-- Open session -->
    <Waitroom v-if="session_status === 'open'" />

    <!-- Running session -->
    <MainTabs
      v-if="session_status !== 'open'"
      class="app__tabs"
      v-model="active_category"
    />
    <div v-if="session_status !== 'open'" class="app__grid">
      <!--
        Grid head
      -->
      <div class="app__grid_head">
        <div class="app__phase_infos">
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
        <TimersText />
        <Btn
          class="ready__btn"
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
          class="ready__btn"
          font_size="1.1rem"
          v-if="
            results_available &&
              phase_infos.phase_no + 1 === phase_infos.nb_phases
          "
          @click="goToGameResults"
        >
          Résultats de la partie
        </Btn>
      </div>
      <!--
        Grid main
      -->
      <div class="app__grid_main">
        <Bilans
          v-show="
            results_available &&
              (active_category === 'Home' || active_category === 'Résultats')
          "
        />
        <div class="app__main">
          <PowerPlantsList
            class="app__main_item"
            :show_actions="!results_available"
            v-show="
              (!results_available && active_category === 'Home') ||
                active_category === 'Centrales'
            "
          />
          <BidsList
            class="app__main_item"
            v-show="
              (!results_available && active_category === 'Home') ||
                active_category === 'Marché'
            "
          />
          <Chatroom
            class="app__main_item"
            v-show="active_category === 'Chat'"
            style="padding: 1rem; max-width: 500px;"
          />
        </div>
      </div>
    </div>

    <BilansSimple class="app__footer_bilans" />
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
import TimersText from "./TimersText.vue";
import Waitroom from "./Waitroom.vue";
import MainTabs from "./MainTabs.vue";

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
    Btn,
    TimersText,
    Waitroom,
    MainTabs
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

  // Tabs
  active_category = "Home";

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

.app__tabs {
  max-width: 700px;
  margin: auto;
}
.app__phase_infos {
  margin: 1rem;
  font-size: 1.15rem;
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
    height: 3rem;
  }

  .app__tabs {
    width: 90%;
  }
}

@media screen and (max-width: 400px) {
  .app__main_item {
    margin: 1rem 3px;
    border: none;
    padding: 4px;
    position: relative;
  }

  .app__footer_bilans {
    font-size: 1.5rem;
    height: 2.5rem;
  }

  .app__tabs {
    width: 100%;
  }
}

.app__footer_bilans {
  width: 100%;
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
