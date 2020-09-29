<template>
  <div class="bid__list">
    <h2>Bourse de l'électricité</h2>

    <div v-if="dummy || can_bid">
      <h3>Poster une enchère</h3>
      <div class="bid__action" v-if="dummy || can_bid">
        <BidItemInput :type="'buy'" :edit="true" :dummy="dummy" />
      </div>
      <h3 v-if="bidsBuy.length > 0">Vos enchères</h3>
      <BidItem
        :type="'buy'"
        :edit="false"
        v-for="bid in bids_sorted"
        :key="bid.id"
        :bid="bid"
        :dummy="dummy"
      />
      <h3 v-if="bidsBuy.length === 0 && bidsSell.length === 0">
        Vous n'avez pas d'enchères
      </h3>
    </div>

    <div v-if="!dummy && !can_bid && !clearing_available">
      <h3>Clearing en cours ...</h3>
    </div>

    <div v-if="!dummy && clearing_available">
      <h3>Résultats des enchères</h3>
      <p style="font-size: 1.2rem;">
        Prix du marché : <strong>{{ clearing.price_eur_per_mwh }}</strong> €/MWh
      </p>
      <p style="font-size: 1.2rem;">
        Volumes échangés : <strong>{{ clearing.volume_mwh }}</strong> MWh
      </p>
      <h3>Votre position sur le marché</h3>
      <p v-if="sell.volume_mwh > 0" style="font-size: 1.2rem;">
        Vous vendez <strong>{{ sell.volume_mwh }}</strong> MWh à
        <strong>{{ sell.price_eur_per_mwh }}</strong> €/MWh
      </p>
      <p v-if="buy.volume_mwh > 0" style="font-size: 1.2rem;">
        Vous achetez <strong>{{ buy.volume_mwh }}</strong> MWh à
        <strong>{{ buy.price_eur_per_mwh }}</strong> €/MWh
      </p>
      <p
        v-if="sell.volume_mwh === 0 && buy.volume_mwh === 0"
        style="font-size: 1.2rem;"
      >
        Vous n'avez pas d'enchères retenues
      </p>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Bid } from "../store/bids";
import BidItemInput from "./BidItemInput.vue";
import BidItem from "./BidItem.vue";

const userModule = namespace("user");
const sessionModule = namespace("session");
const bidsModule = namespace("bids");

@Component({ components: { BidItemInput, BidItem } })
export default class BidsList extends Vue {
  @Prop({ default: false }) dummy!: boolean;
  @bidsModule.Getter bids!: Bid[];
  @bidsModule.State clearing!: any;
  @bidsModule.State buy!: any;
  @bidsModule.State sell!: any;
  @sessionModule.Getter can_bid!: boolean;
  @sessionModule.Getter clearing_available!: boolean;

  get bids_sorted(): Bid[] {
    return this.bids
      .map((bid) => bid)
      .sort((a, b) => {
        if (a.type !== b.type) {
          return a.type < b.type ? -1 : 1;
        } else {
          return a.price_eur_per_mwh - b.price_eur_per_mwh;
        }
      });
  }

  get bidsSell(): Bid[] {
    return this.bids.filter((bid) => bid.type === "sell");
  }

  get bidsBuy(): Bid[] {
    return this.bids.filter((bid) => bid.type === "buy");
  }
}
</script>

<style scoped>
.bid__action {
  margin: 0 3px;
}

.bid__list {
  border-radius: 2px;
  overflow: hidden;
}

@media screen and (min-width: 400px) {
  .bid__list {
    border: 2px solid gray;
  }
}

@media screen and (max-width: 400px) {
  .bid__list {
    border: 1px solid gray;
  }
}

.bid__list h3 {
  text-align: start;
  padding-left: 2rem;
}

.bid__list_item {
  margin: 1rem;
}
</style>
