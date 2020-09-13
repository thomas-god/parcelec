<template>
  <div class="app__grid">
    <AuctionSelect v-if="!auction.id" class="app__grid_main" />
    <UsernameSelect v-if="auction.id && !username" class="app__grid_main" />
    <h1 class="app__grid_main" v-if="auction.id && username">
      Hello {{ username }}, bienvenue sur l'enchère {{ auction.name }} !
    </h1>
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
  grid-template-rows: 50px 1fr;
  grid-template-columns: 2fr 1fr;
}

.app__grid_main {
  grid-area: main;
}

.chatroom {
  grid-area: message;
}
</style>
