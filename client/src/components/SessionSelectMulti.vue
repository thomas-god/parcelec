<template>
  <div>
    <h1>Rejoindre une partie existante</h1>
    <ul class="sessions_list">
      <li v-for="s in open_sessions" :key="s.name" @click="goToSession(s.id)">
        <span>{{ s.name }}</span>
      </li>
      <li v-if="open_sessions.length === 0">
        Il n'y a pas de partie à rejoindre
      </li>
    </ul>
  </div>
</template>

<script lang="ts">
import { Component, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Session } from "../store/session";

const sessionModule = namespace("session");

@Component
export default class SessionSelectMulti extends Vue {
  // Store related stuff
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

  async created(): Promise<void> {
    await this.getOpenSessions();
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
  font-size: 1.3rem;
}

li:hover span {
  background-color: rgb(0, 151, 98);
}
</style>
