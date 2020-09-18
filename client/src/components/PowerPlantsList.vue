<template>
  <div class="pp__list">
    <h2>Votre parc de production</h2>
    <PowerPlantItem
      class="pp__list_item"
      v-for="pp in pp_sorted"
      :key="`pp-list-${pp.id}`"
      :power_plant="pp"
      :power_max_mw="power_plants_max_power_mw"
    />
    <div class="actions">
      <button @click="updatePlanning">Envoyer</button>
      <button @click="resetPlanning">Reset</button>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { PowerPlant } from "../store/portfolio";
import PowerPlantItem from "./PowerPlantItem.vue";

const userModule = namespace("user");
const sessionModule = namespace("session");
const portfolioModule = namespace("portfolio");

@Component({ components: { PowerPlantItem } })
export default class PowerPlantsList extends Vue {
  @portfolioModule.Getter power_plants!: PowerPlant[];
  @portfolioModule.Action resetPlanning!: () => {};
  @portfolioModule.Action onSuccessfulPlanningUpdate!: () => {};
  @State api_url!: string;
  @userModule.State user_id!: string;
  @sessionModule.Getter session_id!: string;

  get pp_sorted(): PowerPlant[] {
    return this.power_plants
      .map((pp) => pp)
      .sort((a, b) => b.p_max_mw - a.p_max_mw);
  }

  get power_plants_max_power_mw(): number {
    return this.pp_sorted[0].p_max_mw;
  }

  async updatePlanning() {
    const planning_formatted = this.power_plants.map((pp) => {
      return {
        user_id: this.user_id,
        session_id: this.session_id,
        plant_id: pp.id,
        p_mw: pp.planning_modif,
      };
    });
    const res = await fetch(
      `${this.api_url}/session/${this.session_id}/user/${this.user_id}/planning`,
      {
        method: "PUT",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(planning_formatted),
      }
    );
    if (res.status === 201) {
      this.onSuccessfulPlanningUpdate();
    } else {
      console.log(await res.text());
    }
  }
}
</script>

<style scoped>
.pp__list_item {
  margin: 1rem 1.5rem 1rem 1rem;
}

.actions {
  margin: 1rem;
}
.actions button {
  font-size: 1rem;
  margin: 0 1rem;
}
</style>