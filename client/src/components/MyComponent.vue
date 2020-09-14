<template>
  <div v-if="!(auction.id && username)" class="app_full">
    <h1>Bienvenue sur Parcélec ! ⚡️</h1>
    <AuctionSelect v-if="!auction.id" />
    <UsernameSelect v-if="auction.id && !username" />
  </div>
  <div v-else class="app_full">
    <div class="app__grid_main">
      <!-- <Bid v-if="auction.id && username" /> -->
    </div>
    <h1 class="app_full">
      Bonjour {{ username }}, vous avez rejoint la partie
      <em>{{ auction.name }}</em> !
    </h1>
    <h3 class="app_full">
      Vous pouvez discuter avec les autres joueurs connectés, et quand vous
      serez prêt à démarrer la partie, cliquez sur le bouton
      <em>"Je suis prêt!"</em>
    </h3>
    <Chatroom class="chatroom" v-if="auction.id && username" />
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Auction } from "../store/auction";
import AuctionSelect from "./AuctionSelect.vue";
import UsernameSelect from "./UsernameSelect.vue";
import Chatroom from "./Chatroom.vue";
import Messages from "./Messages.vue";

const userModule = namespace("user");
const auctionModule = namespace("auction");

@Component({
  components: { AuctionSelect, UsernameSelect, Chatroom },
})
export default class HelloWorld extends Vue {
  @userModule.Getter username!: string;
  @auctionModule.Getter auction!: Auction;
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
  margin: auto;
}

.app_full h3 {
  max-width: 650px;
  margin: auto;
  margin-bottom: 2rem;
}

.chatroom {
  width: 85vw;
  margin: auto;
}
</style>
