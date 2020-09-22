<template>
  <div>
    <h2>Nouvelle partie</h2>

    <h3>Choisissez un scenario ...</h3>
    <ul class="sessions_list">
      <li v-for="s in scenarios" :key="s.name" @click="scenario_id = s.id">
        <span :class="s.id === scenario_id ? 'scenario__selected' : ''">{{
          s.name
        }}</span>
      </li>
    </ul>
    <div class="session_open">
      <label for="session_open_input">
        <h3>... et choisissez un nom de partie</h3>
      </label>
      <div>
        <input
          type="text"
          v-model="new_session_name"
          v-on:keyup.enter="openSession()"
          id="session_open_input"
        />
        <button @click="openSession()">Open</button>
      </div>
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

const sessionModule = namespace("session");

@Component
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
.sessions_list {
  max-width: 200px;
  margin: auto;
  margin-bottom: 1.5rem;
  padding: 1rem;
  border: 1px solid rgba(0, 0, 0, 0.493);
  border-radius: 3px;
  box-shadow: 12px 12px 2px 1px rgba(28, 28, 56, 0.26);
}

ul {
  list-style-type: none;
  padding: 0;
}

li span {
  padding: 0 0.5rem;
}

li:hover span,
.scenario__selected {
  background-color: rgb(0, 151, 98);
}

.session_open {
  display: flex;
  flex-direction: column;
  margin: auto;
}
</style>
