<template>
  <div :class="session.multi_game ? 'app__waitroom' : 'app__waitroom_solo'">
    <div class="app__waitroom__title">
      <h1>
        Bonjour {{ username }}, vous avez rejoint la partie
        <em>{{ session.name }}</em> !
      </h1>
      <p>
        Vous pouvez
        {{
          session.multi_game
            ? "discuter avec les autres joueurs connectés,"
            : ""
        }}
        prendre connaissance de vos centrales, et quand vous serez prêt·e à
        démarrer la partie, cliquez sur le bouton
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
</template>

<script lang="ts">
import { Component, Vue } from "vue-property-decorator";
import { State, namespace } from "vuex-class";
import { Session } from "../store/session";
import Chatroom from "./Chatroom.vue";
import PowerPlantsList from "./PowerPlantsList.vue";
import Btn from "./base/Button.vue";

const userModule = namespace("user");
const sessionModule = namespace("session");
const portfolioModule = namespace("portfolio");
const resultsModule = namespace("results");

@Component({
  components: {
    Chatroom,
    PowerPlantsList,
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
  @sessionModule.Getter phase_infos!: Session["phase_infos"];

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
.main {
  height: calc(100%-36px);
  margin-bottom: 4.5rem;
}

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
</style>
