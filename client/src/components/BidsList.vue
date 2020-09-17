<template>
  <div class="bid__list">
    <h2>Vos enchères</h2>
    <div class="bid__action">
      <BidItem :type="'buy'" :edit="true" />
      <BidItem :type="'sell'" :edit="true" />
    </div>
    <h3>Vos achats</h3>
    <BidItem
      :type="'buy'"
      :edit="false"
      v-for="bid in bidsBuy"
      :key="bid.id"
      :bid="bid"
    />
    <h3>Vos ventes</h3>
    <BidItem
      :type="'sell'"
      :edit="false"
      v-for="bid in bidsSell"
      :key="bid.id"
      :bid="bid"
    />
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Bid } from "../store/bids";
import BidItem from "./BidItem.vue";

const userModule = namespace("user");
const sessionModule = namespace("session");
const bidsModule = namespace("bids");

@Component({ components: { BidItem } })
export default class BidsList extends Vue {
  @bidsModule.Getter bids!: Bid[];

  get bidsSell(): Bid[] {
    return this.bids.filter(bid => bid.type === "sell");
  }

  get bidsBuy(): Bid[] {
    return this.bids.filter(bid => bid.type === "buy");
  }
}
</script>

<style scoped>
.bid__action {
  border: 2px solid grey;
  margin: 1rem;
}

.bid__list {
  max-width: 400px;
  border: 2px solid gray;
  border-radius: 2px;
}

.bid__list h3 {
  text-align: start;
  padding-left: 2rem;
}

.bid__list_item {
  margin: 1rem;
}
</style>