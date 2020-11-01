<template>
  <div class="main" id="main">
    <!-- Tabs -->
    <MainTabs class="tabs" v-model="active_tab" :tabs="tabs" />

    <!-- Content -->
    <div class="content">
      <MainWaitroom class="content_item wide" v-show="show_waitroom" />
      <MainInfos class="content_item" v-show="show_phase_infos" />
      <PowerPlantsList
        class="content_item card"
        :show_actions="!results_available"
        v-show="show_pp_list"
      />
      <Forecast class="content_item card" v-show="show_forecast" />
      <BidsList class="content_item card" v-show="show_bids" />
      <OTC class="content_item card" v-show="show_otcs" />
      <Bilans class="content_item card" v-show="show_results" />
      <Chatroom
        class="content_item card"
        v-show="show_chatroom"
        :display_ready="session.status === 'open'"
      />
    </div>

    <!-- Footer -->
    <BilansSimple class="footer" />
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
import MainWaitroom from "./MainWaitroom.vue";
import MainInfos from "./MainInfos.vue";
import MainTabs from "./MainTabs.vue";
import Forecast from "./Forecast.vue";
import OTC from "./OTC.vue";

const user_module = namespace("user");
const session_module = namespace("session");
const portfolio_module = namespace("portfolio");
const results_module = namespace("results");

@Component({
  components: {
    Chatroom,
    Bid,
    PowerPlantsList,
    BidsList,
    BilansSimple,
    Bilans,
    Btn,
    MainWaitroom,
    MainInfos,
    MainTabs,
    Forecast,
    OTC
  }
})
export default class Main extends Vue {
  @State("api_url") api_url!: string;
  @user_module.Getter username!: string;
  @user_module.Getter user_id!: string;
  @user_module.State ready!: boolean;
  @user_module.Mutation SET_GAME_READY!: (game_ready: boolean) => void;
  @session_module.Getter session!: Session;
  @session_module.Getter session_status!: string;
  @session_module.Getter phase_infos!: Session["phase_infos"];
  @session_module.Getter session_id!: string;
  @portfolio_module.Getter conso!: number;
  @portfolio_module.Getter conso_forecast!: number[];

  // Abilities booleans
  @session_module.Getter can_bid!: boolean;
  @session_module.Getter can_post_planning!: boolean;
  @session_module.Getter clearing_available!: boolean;
  @session_module.Getter results_available!: boolean;

  /**
   * Dynamic tabs
   */
  active_tab = "Home";
  get tabs(): string[] {
    const tabs = ["Home"];
    tabs.push("Centrales");
    if (this.session.status !== "open") tabs.push("Marché");
    if (this.conso_forecast.length > 0) tabs.push("Prévisions");
    if (this.session.multi_game) tabs.push("Chat");
    if (this.session.results_available) tabs.push("Résultats");
    return tabs;
  }

  /**
   * Display flags
   */
  get show_waitroom(): boolean {
    return this.session.status === "open" && this.active_tab === "Home";
  }
  get show_phase_infos(): boolean {
    return this.session.status !== "open";
  }
  get show_pp_list(): boolean {
    return (
      (this.session.status === "open" && this.active_tab === "Centrales") ||
      (this.session.status !== "open" &&
        (this.active_tab === "Centrales" ||
          (!this.session.results_available && this.active_tab === "Home")))
    );
  }
  get show_forecast(): boolean {
    return this.active_tab === "Prévisions";
  }
  get show_bids(): boolean {
    return this.session.status !== "open" && this.active_tab === "Marché";
  }
  get show_otcs(): boolean {
    return (
      this.session.status !== "open" &&
      this.session.multi_game &&
      this.active_tab === "Marché"
    );
  }
  get show_results(): boolean {
    return (
      this.results_available && ["Home", "Résultats"].includes(this.active_tab)
    );
  }
  get show_chatroom(): boolean {
    return this.session.multi_game && this.active_tab === "Chat";
  }

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

.tabs {
  max-width: 700px;
  margin: auto;
  margin-bottom: 2rem;
}

.content {
  width: 100%;
  max-width: 700px;
  margin: auto;
  display: flex;
  flex-direction: column;
  flex-wrap: wrap;
  align-items: center;
  justify-content: flex-start;
}

.content_item {
  width: 100%;
  max-width: 600px;
  box-sizing: border-box;
}

.footer {
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

@media screen and (min-width: 500px) {
  .content_item {
    margin-bottom: 1.5rem;
    border-radius: 2px;
    padding: 0 10px 10px 10px;
  }
  .card {
    margin-top: 0rem;
    padding-top: 10px;
    max-width: 500px !important;
    border: 2px solid gray;
  }
  .card:first-child {
    margin-top: 2rem !important;
  }
  .wide {
    max-width: 700px;
  }
  .footer {
    font-size: 2rem;
    height: 3rem;
  }
  .tabs {
    width: 90%;
  }
}

@media screen and (max-width: 500px) {
  .content_item {
    margin: 1rem 3px;
    border: none;
    padding: 4px;
    position: relative;
  }
  .footer {
    font-size: 1.5rem;
    height: 2.5rem;
  }
  .tabs {
    width: 100%;
  }
}
</style>
