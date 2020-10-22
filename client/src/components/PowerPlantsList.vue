<template>
  <div class="pp__list">
    <h2 v-if="show_title">Vos centrales</h2>
    <PowerPlantItem
      class="pp__list_item"
      v-for="(pp, id) in pp_sorted"
      :class="id !== pp_sorted.length - 1 ? 'pp__list_item_w_separator' : ''"
      :key="`pp-list-${pp.id}`"
      :power_plant="pp"
      :power_max_mw="power_plants_max_power_mw"
      :editable="dummy || can_post_planning"
    />
    <div class="actions" v-if="show_actions">
      <Btn
        @click="updatePlanning"
        :disabled="btn_disabled"
        :background_color="btn_disabled ? 'rgb(0, 132, 255)' : 'red'"
      >
        {{ envoyer_btn_txt }}
      </Btn>
      <Btn
        @click="resetPlanning"
        :disabled="btn_disabled"
        background_color="rgba(0, 132, 255, 0.8)"
      >
        Effacer
      </Btn>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { PowerPlant } from "../store/portfolio";
import PowerPlantItem from "./PowerPlantItem.vue";
import Btn from "../components/base/Button.vue";

const userModule = namespace("user");
const sessionModule = namespace("session");
const portfolioModule = namespace("portfolio");

@Component({ components: { PowerPlantItem, Btn } })
export default class PowerPlantsList extends Vue {
  @Prop({ default: true }) show_actions!: boolean;
  @Prop({ default: false }) dummy!: boolean;
  @Prop({ default: true}) show_title!: boolean;
  @portfolioModule.Getter power_plants!: PowerPlant[];
  @portfolioModule.Action resetPlanning!: () => {};
  @portfolioModule.Action onSuccessfulPlanningUpdate!: () => {};
  @State api_url!: string;
  @userModule.State user_id!: string;
  @sessionModule.Getter session_id!: string;
  @sessionModule.Getter can_post_planning!: boolean;
  envoyer_btn_txt = "Envoyer";

  get pp_sorted(): PowerPlant[] {
    return this.power_plants
      .map(pp => pp)
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
      const planning_formatted = this.power_plants.map(pp => {
        return {
          user_id: this.user_id,
          session_id: this.session_id,
          plant_id: pp.id,
          p_mw: pp.planning_modif
        };
      });
      const res = await fetch(
        `${this.api_url}/session/${this.session_id}/user/${this.user_id}/planning`,
        {
          method: "PUT",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(planning_formatted)
        }
      );
      if (res.status === 201) {
        this.onSuccessfulPlanningUpdate();
        this.envoyer_btn_txt = "OK!";
        setTimeout(() => {
          this.envoyer_btn_txt = "Envoyer";
        }, 500);
      } else {
        console.log(await res.text());
      }
    } else {
      this.envoyer_btn_txt = "OK!";
      setTimeout(() => {
        this.envoyer_btn_txt = "Envoyer";
      }, 500);
      this.onSuccessfulPlanningUpdate();
    }
  }

  get btn_disabled(): boolean {
    return (
      (!this.dummy && !this.can_post_planning) || this.diff_abs_planning === 0
    );
  }
}
</script>

<style scoped>
.pp__list {
  overflow: hidden;
}

.pp__list h2 {
  margin-top: 0;
}

@media screen and (min-width: 400px) {
  .pp__list_item {
    margin: 1rem 1.5rem;
  }
}
@media screen and (max-width: 400px) {
  .pp__list_item {
    position: relative;
    margin: 0 1.5rem 1.5rem;
    padding-bottom: 1.5rem;
  }

  .pp__list_item_w_separator::after {
    content: "";
    position: absolute;
    bottom: 0;
    left: 2.5%;
    width: 95%;
    height: 1px;
    border-bottom: 2px solid rgba(128, 128, 128, 0.466);
  }
}

.actions {
  display: flex;
  flex-direction: row;
  justify-content: center;
  margin: 1rem;
}
.actions button {
  margin: 0 1rem;
}
</style>
