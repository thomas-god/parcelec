<template>
  <div>
    <AuctionSelect v-if="!auction.id" />
    <UsernameSelect v-if="auction.id && !username" />
    <div v-if="auction.id && username">
      <p>Hello {{ username }}, bienvenu sur l'enchère {{ auction.name }} !</p>
      <AuctionSalon />
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Auction } from "../store/auction";
import AuctionSelect from "./AuctionSelect.vue";
import UsernameSelect from "./UsernameSelect.vue";
import AuctionSalon from "./AuctionSalon.vue";
import Messages from "./Messages.vue";

const userModule = namespace("user");
const auctionModule = namespace("auction");

@Component({
  components: { AuctionSelect, UsernameSelect, AuctionSalon }
})
export default class HelloWorld extends Vue {
  @userModule.Getter username!: string;
  @auctionModule.Getter auction!: Auction;
}
</script>
