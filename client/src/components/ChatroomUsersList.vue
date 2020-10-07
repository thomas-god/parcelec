<template>
  <div class="container">
    <h3>Joueurs connectés</h3>
    <ul>
      <li v-for="user in users" :key="user.name">
        {{ user.name }}
        <span v-if="display_ready_status && user.ready">✅</span>
      </li>
    </ul>
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
}
</script>

<style scoped>
h3 {
  margin-top: 10px;
  margin-bottom: 0;
}
ul {
  margin-top: 0;
  margin-bottom: 0rem;
}
li {
  text-align: start !important;
  font-size: 1.1rem;
}

.container {
  display: grid;
  grid-template-rows: auto auto;
  align-items: center;
}

</style>
