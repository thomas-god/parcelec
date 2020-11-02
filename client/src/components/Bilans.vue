<template>
  <div class="bilans__main">
    <div
      class="bilans__ranking"
      v-if="results_available && session_nb_users > 1"
    >
      <h3>
        Classement
      </h3>

      <ToggleSwitch
        v-model="ranking_type"
        :left_label="'Phase'"
        :left_value="'phase'"
        :right_label="'Total'"
        :right_value="'overall'"
        class="bilans__ranking__toggle"
      />

      <template v-for="user in ranking_current">
        <div class="bilans__ranking__item" :key="`ranking-${user.username}`">
          <span class="bilans__ranking__rank">{{ user.rank }}.</span>
          <span class="bilans__ranking__username">
            <strong>{{ user.username }}</strong>
          </span>
          <span class="bilans__ranking__balance"
            >{{ Math.floor(user.balance).toLocaleString("fr-FR") }}€</span
          >
        </div>
      </template>
    </div>

    <div v-else>
      <h3>Gains nets</h3>
      <span class="balance">{{ Math.floor(balance_eur).toLocaleString('fr-FR') }} €</span>
    </div>
    <div class="graph">
      <h3>Détails de vos résultats</h3>
      <MainDataFinances :display_title="false" />
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { BidsState, EnergyExchange } from "../store/bids";
import { PowerPlant } from "../store/portfolio";
import { ResultsState } from "../store/results";
import Btn from "./base/Button.vue";
import ToggleSwitch from "./base/Toggle.vue";
import MainDataFinances from "./MainDataFinances.vue";

const portfolioModule = namespace("portfolio");
const bidsModule = namespace("bids");
const resultsModule = namespace("results");
const sessionModule = namespace("session");

@Component({ components: { Btn, ToggleSwitch, MainDataFinances } })
export default class Bilans extends Vue {
  @sessionModule.State results_available!: boolean;
  @portfolioModule.State power_plants!: PowerPlant[];
  @portfolioModule.State conso!: number;
  @resultsModule.State conso_eur!: number;
  @resultsModule.State prod_eur!: number;
  @resultsModule.State sell_eur!: number;
  @resultsModule.State sell_mwh!: number;
  @resultsModule.State buy_eur!: number;
  @resultsModule.State buy_mwh!: number;
  @resultsModule.State imbalance_costs_eur!: number;
  @resultsModule.State balance_eur!: number;
  @resultsModule.State rankings!: ResultsState["rankings"];

  // Ranking
  @sessionModule.Getter session_nb_users!: number;
  @resultsModule.Getter user_rankings!: number;

  /**
   * Sorted rankings
   */
  ranking_type: "phase" | "overall" = "phase";
  get ranking_phase_sorted() {
    return this.rankings.phase.map(u => u).sort((a, b) => a.rank - b.rank);
  }
  get ranking_overall_sorted() {
    return this.rankings.overall.map(u => u).sort((a, b) => a.rank - b.rank);
  }
  get ranking_current() {
    return this.ranking_type === "phase"
      ? this.ranking_phase_sorted
      : this.ranking_overall_sorted;
  }

  /**
   * Totals
   */
  get deficit_mwh() {
    return Number(
      this.buy_mwh + this.prod_total_mwh - this.conso - this.sell_mwh
    );
  }
  get money() {
    return Number(
      Number(this.conso_eur) +
        Number(this.sell_eur) -
        Number(this.prod_eur) -
        Number(this.buy_eur) +
        Number(this.imbalance_costs_eur)
    );
  }
  get prod_total_mwh(): number {
    let prod = 0;
    if (this.power_plants.length > 0) {
      prod = this.power_plants
        .map(pp => Number(pp.planning_modif))
        .reduce((a, b) => a + b);
    }
    return prod;
  }

  fmt(nb: number): string {
    return Math.floor(nb).toLocaleString("fr-FR");
  }
}
</script>

<style scoped>
.bilans__main h2 {
  margin-top: 0;
}
.bilans__main h3 {
  margin-top: 0;
  margin-bottom: 10px;
}
.bilans__ranking {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-bottom: 1rem;
}
.bilans__ranking__toggle {
  max-width: 400px;
  width: 70%;
  margin: 0 0 10px;
}
.bilans__ranking__item {
  width: 100%;
  max-width: 300px;
  box-sizing: border-box;
  display: grid;
  grid-template-areas: "rank username balance";
  grid-template-columns: 20px auto 70px;
  grid-template-rows: 2rem;
  align-items: center;

  margin: 0.3rem 1rem;
  padding: 5px 1rem;
  border: 1px solid rgb(214, 214, 214);
  border-radius: 0.7rem;
  -webkit-box-shadow: -4px 4px 10px -8px rgba(112, 112, 112, 1);
  -moz-box-shadow: -4px 4px 10px -8px rgba(112, 112, 112, 1);
  box-shadow: -4px 4px 10px -8px rgba(112, 112, 112, 1);
}
.bilans__ranking__rank {
  grid-area: rank;
}
.bilans__ranking__username {
  grid-area: username;
  text-align: start;
  padding-left: 10px;
  overflow: hidden;
}
.bilans__ranking__balance {
  grid-area: balance;
  text-align: end;
}

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

.graph {
  margin-top: 2rem;
}
.balance {
  font-size: 2rem;
}
</style>
