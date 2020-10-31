<template>
  <div>
    <p>
      Le marché est un lieu où les différents joueurs d'une partie peuvent
      s'échanger de l'énergie : il permet aux joueurs de trouver l'énergie qui
      leur manque pour être à l'équilibre ou bien de vendre leur production
      excédentaire.
    </p>
    <p>
      Les joueurs peuvent faire une ou plusieurs offres : <em>acheter</em> ou
      <em>vendre</em> un <em>volume</em> d'énergie (en MWh) à un certain
      <em>prix</em> (en €/MWh).
    </p>

    <bids-list class="card" :dummy="true" />

    <p>
      Une fois le marché fermé toutes les offres sont comparées pour mettre en
      relation un vendeur et un acheteur : pour qu'un échange se fasse il faut
      un <em>acheteur disposé à payer le prix exigé par un vendeur</em>. Une
      fois les enchères comparées vous saurez lesquelles de vos offres auront
      été retenues. Attention, il vous faudra peut être ajuster votre programme
      d'appel en conséquence !
    </p>
    <p>
      Une fois le marché fermé vous aurez accès à toutes les offres
      (anonymisées) qui auront été faites : étudier ces offres peut vous être
      utile pour comprendre les stratégies de vos concurrents !
    </p>
    <BidsGraphAll class="chart card" />
  </div>
</template>

<script lang='ts'>
import { Component, Vue } from "vue-property-decorator";
import { namespace } from "vuex-class";
import BidsList from "../components/BidsList.vue";
import BidsGraphAll from "./BidsGraphAll.vue";

const bids_module = namespace("bids");

@Component({ components: { BidsList, BidsGraphAll } })
export default class TutoMarket extends Vue {
  @bids_module.Mutation SET_ALL_MARKET_BIDS!: (bids: any[]) => void;
  created() {
    this.SET_ALL_MARKET_BIDS([
      {
        type: "sell",
        volume_mwh: 100,
        price_eur_per_mwh: 10
      },
      {
        type: "sell",
        volume_mwh: 60,
        price_eur_per_mwh: 15
      },
      {
        type: "sell",
        volume_mwh: 50,
        price_eur_per_mwh: 45
      },
      {
        type: "buy",
        volume_mwh: 150,
        price_eur_per_mwh: 40
      },
      {
        type: "buy",
        volume_mwh: 50,
        price_eur_per_mwh: 20
      }
    ]);
  }
}
</script>

<style scoped>
.card {
  padding: 1rem;
  border: 1px solid rgba(0, 0, 0, 0.493);
  border-radius: 3px;
  width: 100%;
  max-width: 500px;
  box-sizing: border-box;
  margin: auto;
}
.chart {
  position: relative;
  max-width: 100%;
  height: 300px;
  padding: 5px 10px 5px 0;
}

@media screen and (min-width: 400px) {
  .chart {
    width: 80vw;
  }
}

@media screen and (max-width: 400px) {
  .chart {
    width: 95vw;
  }
}
</style>