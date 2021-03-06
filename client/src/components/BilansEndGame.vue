<template>
  <div>
    <h1>Résultats de la partie</h1>
    <h2 v-if="session_nb_users > 1">
      Classement : {{ rank_final }} / {{ session_nb_users }}
    </h2>
    <h3>Gains finaux : {{ fmt(money_total) }} €</h3>
    <div v-for="(phase, idx) in results" :key="`res-phase-${idx}`">
      <h3>Phase {{ phase.phase_no + 1 }}</h3>
      <div class="bilans__container">
        <div class="bilans__item">
          <span><em>Clients</em></span>
          <span> {{ fmt(phase.conso_eur) }} € </span>
        </div>
        <div class="bilans__item">
          <span><em>Production</em></span>
          <span> {{ fmt(-1 * phase.prod_eur) }} € </span>
        </div>
        <div class="bilans__item">
          <span><em>Ventes</em></span>
          <span> {{ fmt(phase.sell_eur) }} € </span>
        </div>
        <div class="bilans__item">
          <span><em>Achats</em></span>
          <span> {{ fmt(-1 * phase.buy_eur) }} € </span>
        </div>
        <div class="bilans__item">
          <span><em>Écarts</em></span>
          <span> {{ fmt(phase.imbalance_costs_eur) }} € </span>
        </div>
        <div class="bilans__item">
          <span>Total</span>
          <span> {{ fmt(phase.balance_eur) }} € </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { BidsState, EnergyExchange } from "../store/bids";
import { PowerPlant } from "../store/portfolio";

const portfolioModule = namespace("portfolio");
const userModule = namespace("user");
const resultsModule = namespace("results");
const sessionModule = namespace("session");

@Component
export default class BilansEndGame extends Vue {
  @State("api_url") api_url!: string;
  @sessionModule.Getter session_id!: string;
  @sessionModule.Getter nb_phases!: number;
  @sessionModule.Getter session_nb_users!: number;
  @userModule.Getter user_id!: string;

  results: { phase_no: number; ranking_overall: number }[] = [];
  async mounted() {
    const res = await fetch(
      `${this.api_url}/session/${this.session_id}/user/${this.user_id}/game_results`,
      { method: "GET" }
    );
    if (res.status === 200) {
      this.results = await res.json();
    } else {
      console.log(await res.text());
      this.results = [];
    }
  }

  get money_total(): number {
    return this.results.reduce((a, b: any) => a + b.balance_eur, 0 as number);
  }

  get rank_final(): number {
    const tmp = this.results.find(res => res.phase_no === this.nb_phases - 1);
    return tmp === undefined ? -1 : tmp.ranking_overall;
  }

  fmt(nb: number): string {
    return Math.round(nb).toLocaleString("fr-FR");
  }
}
</script>

<style scoped>
.bilans__container {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  justify-content: center;
  align-items: center;
}

.bilans__container .bilans__item:last-of-type span {
  font-weight: 500;
}

.bilans__item {
  display: flex;
  flex-direction: column;
  margin: 1rem;
  flex-basis: auto;
  flex-grow: 0;
}
.bilans__item span {
  justify-self: start;
  text-align: center;
  white-space: nowrap;
}
</style>
