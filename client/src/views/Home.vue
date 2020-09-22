<template>
  <div class="home">
    <h1>Bienvenue sur Parcélec ! ⚡️</h1>
    <p>
      Lorem ipsum dolor sit amet consectetur adipisicing elit. Dignissimos id
      fugit provident vel odit minima ab possimus alias facere! Nobis
      consequuntur recusandae veniam iste accusantium nulla placeat quo pariatur
      nam.
    </p>
    <p>
      Lorem ipsum dolor sit amet consectetur adipisicing elit. Dignissimos id
      fugit provident vel odit minima ab possimus alias facere! Nobis
      consequuntur recusandae veniam iste accusantium nulla placeat quo pariatur
      nam.
    </p>
    <h2>Vos centrales</h2>
    <div class="grid__left">
      <power-plants-list class="card" :dummy="true" />
      <div>
        <p>
          Lorem, {{ prod_mw }} ipsum dolor sit amet consectetur adipisicing
          elit. Iure eligendi explicabo facere accusantium necessitatibus ab
          deserunt vel doloribus, dolores aperiam quaerat distinctio dolore eos
          temporibus qui sapiente mollitia enim perspiciatis!
        </p>
        <p>
          Lorem, ipsum dolor sit amet consectetur adipisicing elit. Iure
          eligendi explicabo facere accusantium necessitatibus ab deserunt vel
          doloribus, dolores aperiam quaerat distinctio dolore eos temporibus
          qui sapiente mollitia enim perspiciatis!
        </p>
        <p>
          Lorem, ipsum dolor sit amet consectetur adipisicing elit. Iure
          eligendi explicabo facere accusantium necessitatibus ab deserunt vel
          doloribus, dolores aperiam quaerat distinctio dolore eos temporibus
          qui sapiente mollitia enim perspiciatis!
        </p>
      </div>
    </div>
    <h2>Le marché</h2>
    <div class="grid__right">
      <div>
        <p>
          Lorem, ipsum dolor sit amet consectetur adipisicing elit. Iure
          eligendi explicabo facere accusantium necessitatibus ab deserunt vel
          doloribus, dolores aperiam quaerat distinctio dolore eos temporibus
          qui sapiente mollitia enim perspiciatis!
        </p>
        <p>
          Lorem, ipsum dolor sit amet consectetur adipisicing elit. Iure
          eligendi explicabo facere accusantium necessitatibus ab deserunt vel
          doloribus, dolores aperiam quaerat distinctio dolore eos temporibus
          qui sapiente mollitia enim perspiciatis!
        </p>
        <p>
          Lorem, ipsum dolor sit amet consectetur adipisicing elit. Iure
          eligendi explicabo facere accusantium necessitatibus ab deserunt vel
          doloribus, dolores aperiam quaerat distinctio dolore eos temporibus
          qui sapiente mollitia enim perspiciatis!
        </p>
      </div>
      <bids-list class="card" :dummy="true" />
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import PowerPlantsList from "../components/PowerPlantsList.vue";
import BidsList from "../components/BidsList.vue";
import { PowerPlant } from "../store/portfolio";

const portfolioModule = namespace("portfolio");

@Component({ components: { PowerPlantsList, BidsList } })
export default class Home extends Vue {
  @portfolioModule.Mutation SET_POWER_PLANTS!: (
    power_plants: PowerPlant[]
  ) => void;
  @portfolioModule.State power_plants!: PowerPlant[];

  get prod_mw() {
    return this.power_plants.reduce(
      (a, b) => a + Number(b.planning_modif),
      0 as number
    );
  }

  created() {
    // Set dummy portfolio for demonstration purpose
    this.SET_POWER_PLANTS([
      {
        id: "1",
        type: "nuc",
        p_min_mw: 500,
        p_max_mw: 1300,
        stock_max_mwh: -1,
        price_eur_per_mwh: 25,
        planning: 0,
        planning_modif: 0,
        stock_mwh: -1,
      },
      {
        id: "2",
        type: "hydro",
        p_min_mw: 50,
        p_max_mw: 500,
        stock_max_mwh: 500,
        price_eur_per_mwh: 0,
        planning: 0,
        planning_modif: 0,
        stock_mwh: 500,
      },
    ]);
  }
}
</script>

<style scoped>
.home {
  padding: 0 1.5rem;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.home p {
  font-size: 1.3rem;
  max-width: 550px;
  text-align: start;
  margin: 0 1rem 1rem;
}

.card {
  margin: auto;
  margin: 0 1rem 2.5rem 1rem;
  padding: 1rem;
  border: 1px solid rgba(0, 0, 0, 0.493);
  border-radius: 3px;
  box-shadow: 12px 12px 2px 1px rgba(28, 28, 56, 0.26);
  min-width: 400px;
}

.grid__left {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  flex: 1 1 0;
  align-items: center;
  justify-content: center;
}
.grid__left p {
  padding-left: 1rem;
}

.grid__right {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap-reverse;
  flex: 1 1 0;
  align-items: center;
  justify-content: center;
}
.grid__right p {
  text-align: end;
  padding-left: 1rem;
}

@media screen and (max-width: 1110px) {
  .grid__right p {
    text-align: start;
  }
}
</style>
