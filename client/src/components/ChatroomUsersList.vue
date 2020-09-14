<template>
  <div class="container">
    <h3>Joueurs connectés</h3>
    <ul>
      <li v-for="user in users" :key="user.name">
        {{ user.name }}
        <span v-if="display_ready_status && user.ready">✅</span>
      </li>
    </ul>
    <button
      v-if="display_ready_status"
      :disabled="user_ready"
      @click="setStatusReady"
    >
      Je suis pret!
    </button>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { User } from "../store/auction";

const auctionModule = namespace("auction");
const userModule = namespace("user");

@Component
export default class UserList extends Vue {
  @Prop({ default: false }) readonly display_ready_status!: boolean;
  @auctionModule.State users!: User[];
  @auctionModule.Getter auction_id!: string;
  @userModule.Getter user_id!: string;
  @userModule.Getter user_ready!: boolean;
  @userModule.Action setReadyStatus!: () => void;

  async setStatusReady(): Promise<void> {
    const res = await fetch(
      `http://localhost:3000/auction/${this.auction_id}/user_ready`,
      {
        method: "PUT",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ user_id: this.user_id }),
      }
    );
    if (res.status === 201) this.setReadyStatus();
  }
}
</script>

<style scoped>
h3 {
  margin-top: 0;
  margin-bottom: 0;
}
ul {
  margin-bottom: 0;
}
li {
  text-align: start !important;
  font-size: 1.1rem;
}

.container {
  display: grid;
  grid-template-rows: 30px 1fr 40px;
}

button {
  font-size: 1.1rem;
}
</style>
