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
      <h1 class="app__grid_head">Enchère en cours...</h1>
      <div class="app__grid_main">
        <!-- <Bid v-if="session.id && username" /> -->
        <PowerPlantsList v-if="session.id && username" />
      </div>
      <Chatroom class="chatroom__grid" display_direction="column" />
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

const userModule = namespace("user");
const sessionModule = namespace("session");

@Component({
  components: { SessionSelect, UsernameSelect, Chatroom, Bid, PowerPlantsList }
})
export default class Main extends Vue {
  @userModule.Getter username!: string;
  @sessionModule.Getter session!: Session;
  @sessionModule.Getter session_status!: string;
}
</script>

<style scoped>
.app__grid {
  display: grid;
  width: 100%;
  height: 100%;
  grid-template-areas:
    "head head"
    "main  message";
  grid-template-rows: auto 1fr;
  grid-template-columns: 2fr 1fr;
}

.app__grid_head {
  grid-area: head;
  margin-bottom: 1rem;
}

.app__grid_main {
  grid-area: main;
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
</style>
