<template>
  <div class="container">
    <h3>Joueurs connectés</h3>
    <ul>
      <li v-for="user in users" :key="user.name">
        {{ user.name }}
        <span v-if="display_ready_status && user.ready">✅</span>
      </li>
    </ul>
    <Btn
      v-if="display_ready_status"
      :disabled="user_ready"
      @click="setStatusReady"
      font_size="1.2rem"
      background_color="green"
    >
      Je suis prêt·e!
    </Btn>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { User } from "../store/session";
import Btn from './base/Button.vue'

const sessionModule = namespace("session");
const userModule = namespace("user");

@Component({ components: { Btn }})
export default class UserList extends Vue {
  @Prop({ default: false }) readonly display_ready_status!: boolean;
  @sessionModule.State users!: User[];
  @sessionModule.Getter session_id!: string;
  @userModule.Getter user_id!: string;
  @userModule.Getter user_ready!: boolean;
  @userModule.Mutation SET_GAME_READY!: (game_ready: boolean) => void;
  @State("api_url") api_url!: string;

  async setStatusReady(): Promise<void> {
    const res = await fetch(
      `${this.api_url}/session/${this.session_id}/user/${this.user_id}/ready`,
      {
        method: "PUT",
      }
    );
    if (res.status === 201) this.SET_GAME_READY(true);
  }
}
</script>

<style scoped>
h3 {
  margin-top: 0;
  margin-bottom: 0;
}
ul {
  margin-bottom: 2rem;
}
li {
  text-align: start !important;
  font-size: 1.1rem;
}

.container {
  display: grid;
  grid-template-rows: 30px 1fr 40px;
}

</style>
