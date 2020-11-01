<template>
  <div class="container">
    <h1>Nouvelle partie</h1>

    <h2>Choisissez un scenario ...</h2>
    <div class="scenarios__container">
      <ul class="scenarios__list">
        <li
          v-for="s in scenarios"
          :key="s.name"
          @click="getScenarioDetails(s.id)"
          :class="s.id === scenario_id ? 'scenario__selected' : ''"
        >
          {{ s.name }}
        </li>
      </ul>
      <ScenarioIDCard
        :options="scenario_options"
        :portfolio="scenario_portfolio"
        class="scenario__ID"
        v-if="scenario_id !== ''"
      />
    </div>
    <div class="session_open">
      <label for="session_open_input">
        <h2>... et un nom de partie</h2>
      </label>

      <input
        type="text"
        v-model="new_session_name"
        v-on:keyup.enter="openSession()"
        id="session_open_input"
      />
      <Btn @click="openSession()">Créer</Btn>

      <span v-if="new_session_name_err" style="color: red">{{
        new_session_name_err_msg
      }}</span>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Session } from "../store/session";
import ScenarioIDCard from "./SessionSelectScenarioIDCard.vue";
import Btn from "./base/Button.vue";

const sessionModule = namespace("session");

@Component({ components: { ScenarioIDCard, Btn } })
export default class SessionSelectScenario extends Vue {
  // Store related stuff
  @sessionModule.Getter session!: Session;
  @sessionModule.Action setSessionID!: (session_id: string) => void;
  @State("api_url") api_url!: string;

  scenarios = [];
  /**
   * Query the API to get the list of available scenarios.
   */
  async getScenarios(): Promise<void> {
    const res = await fetch(`${this.api_url}/scenarios`, {
      method: "GET"
    });
    this.scenarios = await res.json();
  }

  scenario_id = "";
  scenario_options = {};
  scenario_portfolio = [];
  /** Get details (options, default portofolio of a scenario)
   * @param scenario_id ID of the scenario
   */
  async getScenarioDetails(scenario_id: string): Promise<void> {
    this.scenario_id = scenario_id;
    const res = await fetch(`${this.api_url}/scenario/${this.scenario_id}`, {
      method: "GET"
    });
    if (res.status === 200) {
      const body = await res.json();
      this.scenario_options = body.options;
      this.scenario_portfolio = body.portfolio;
    } else {
      console.log(await res.text());
      this.scenario_options = {};
      this.scenario_portfolio = [];
    }
  }

  // New session name stuff
  new_session_name = "";
  new_session_name_err = false;
  new_session_name_err_msg = "";
  /**
   * Open a new session and store its ID on the store on
   * success.
   */
  async openSession() {
    if (this.new_session_name !== "") {
      const res = await fetch(`${this.api_url}/session/`, {
        method: "PUT",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          session_name: this.new_session_name,
          scenario_id: this.scenario_id
        })
      });
      if (res.status === 201) {
        this.new_session_name_err = false;
        this.new_session_name_err_msg = "";
        const session_id = (await res.json()).id;
        this.goToSession(session_id);
      } else {
        this.new_session_name_err = true;
        this.new_session_name_err_msg = await res.text();
      }
    }
  }

  async created(): Promise<void> {
    await this.getScenarios();
  }

  goToSession(session_id: string): void {
    this.setSessionID(session_id);
    this.$router.push(`/session/${session_id}`);
  }
}
</script>

<style scoped>
.container {
  max-width: 1000px;
  margin: auto;
}

@media screen and (min-width: 750px) {
  .scenarios__container {
    display: grid;
    grid-template-columns: 250px 1fr;
    grid-template-rows: minmax(250px, auto);
    align-items: stretch;
    justify-items: center;
    width: 80%;
    margin: auto;
    gap: 1rem;
  }

  .scenarios__list {
    justify-self: end;
    text-align: start;
  }
  .scenarios__ID {
    justify-self: start;
  }
}
@media screen and (max-width: 750px) {
  .scenarios__container {
    margin: 0 2rem;
    display: grid;
    grid-template-columns: 1fr;
    grid-template-rows: minmax(100px, auto) auto;
    gap: 1rem;
  }
  .scenarios__list {
    text-align: start;
  }
}

.scenarios__list,
.scenario__ID {
  padding: 1rem;
  border: 1px solid rgba(0, 0, 0, 0.493);
  border-radius: 3px;
  height: 100%;
  box-sizing: border-box;
  margin: 0;
  width: 100%;
}

ul {
  list-style-type: "-";
  padding: 0;
}

li {
  padding: 0.2rem 0.5rem;
  margin: 0.3rem 0;
  font-size: 1.1rem;
  box-sizing: border-box;
}

li:hover {
  background-color: rgb(0, 151, 98, 0.2);
}
.scenario__selected {
  background-color: rgba(0, 151, 98, 0.671) !important;
}

.session_open {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin: auto;
}

.session_open button {
  margin-top: 1rem;
  font-size: 1rem;
}

#session_open_input {
  font-size: 1.3rem;
  text-align: center;
}
</style>
