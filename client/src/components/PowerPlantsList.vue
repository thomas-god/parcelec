<template>
  <div class="pp__list">
    <h2>Vos centrales</h2>
    <PowerPlantItem
      class="pp__list_item"
      v-for="pp in pp_sorted"
      :key="`pp-list-${pp.id}`"
      :power_plant="pp"
      :power_max_mw="power_plants_max_power_mw"
      :editable="dummy || can_post_planning"
    />
    <div class="actions" v-if="show_actions">
      <button
        @click="updatePlanning"
        :disabled="(!dummy && !can_post_planning) || diff_abs_planning === 0"
      >
        Envoyer
      </button>
      <button
        @click="resetPlanning"
        :disabled="(!dummy && !can_post_planning) || diff_abs_planning === 0"
      >
        Effacer
      </button>
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
  @Prop({ default: true }) show_actions!: boolean;
  @Prop({ default: false }) dummy!: boolean;
  @portfolioModule.Getter power_plants!: PowerPlant[];
  @portfolioModule.Action resetPlanning!: () => {};
  @portfolioModule.Action onSuccessfulPlanningUpdate!: () => {};
  @State api_url!: string;
  @userModule.State user_id!: string;
  @sessionModule.Getter session_id!: string;
  @sessionModule.Getter can_post_planning!: boolean;

  get pp_sorted(): PowerPlant[] {
    return this.power_plants
      .map((pp) => pp)
      .sort((a, b) => b.p_max_mw - a.p_max_mw);
  }

  get power_plants_max_power_mw(): number {
    return this.pp_sorted[0].p_max_mw;
  }

  get diff_abs_planning(): number {
    return this.power_plants.reduce(
      (a, b) => a + Math.abs(b.planning - b.planning_modif),
      0
    );
  }

  async updatePlanning() {
    if (!this.dummy) {
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
    } else {
      this.onSuccessfulPlanningUpdate();
    }
  }
}
</script>

<style scoped>
.pp__list {
  overflow: hidden;
}

@media screen and (min-width: 400px) {
  .pp__list_item {
    margin: 1rem 1.5rem;
  }
}
@media screen and (max-width: 400px) {
  .pp__list_item {
    margin: 1rem 1.5rem;
  }
}

.actions {
  display: flex;
  flex-direction: row;
  justify-content: center;
  margin: 1rem;
}
.actions button {
  border: none;
  border-radius: 1rem;
  background-color: rgb(0, 132, 255);
  margin: 0 1rem;
  padding: 5px 10px;
  font-size: 1rem;
  font-weight: 600;
  color: white;
}
.actions button:disabled {
  color: rgb(235, 235, 235);
  background-color: rgba(0, 132, 255, 0.616);
}
</style>
