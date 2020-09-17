<template>
  <div>
    <h2>Choisissez une partie à rejoindre</h2>
    <ul class="sessions_list">
      <li v-for="s in open_sessions" :key="s.name" @click="setSessionID(s.id)">
        <span>{{ s.name }}</span>
      </li>
      <li v-if="open_sessions.length === 0">
        Il n'y a pas de partie à rejoindre
      </li>
    </ul>
    <div v-if="allow_new_session" class="session_open">
      <label for="session_open_input">
        Ou bien entrez le nom d'une nouvelle partie :
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
export default class User extends Vue {
  // Store related stuff
  @Prop({ default: false }) allow_new_session!: boolean;
  @sessionModule.Getter session!: Session;
  @sessionModule.Action setSessionID!: (session_id: string) => void;
  @State("api_url") api_url!: string;

  open_sessions: Session[] = [];
  /**
   * Query the API to get the list of open game sessions.
   */
  async getOpenSessions(): Promise<void> {
    const res = await fetch(`${this.api_url}/sessions/open`, {
      method: "GET"
    });
    this.open_sessions = await res.json();
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
    const res = await fetch(`${this.api_url}/session/open`, {
      method: "PUT",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ session_name: this.new_session_name })
    });
    if (res.status === 200) {
      this.new_session_name_err = false;
      this.new_session_name_err_msg = "";
      const session_id = (await res.json()).id;
      this.setSessionID(session_id);
    } else {
      this.new_session_name_err = true;
      this.new_session_name_err_msg = await res.text();
    }
  }

  async created(): Promise<void> {
    await this.getOpenSessions();
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

li:hover span {
  background-color: rgb(0, 151, 98);
}

.session_open {
  display: flex;
  flex-direction: column;
  margin: auto;
}
</style>
