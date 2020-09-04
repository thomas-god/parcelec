<template>
  <div class="user_add_pseudo">
    <label for="user_add_pseudo_input">
      Choisissez un pseudo :
    </label>
    <div>
      <input
        type="text"
        v-model="new_username"
        v-on:keyup.enter="addUsername()"
        id="user_add_pseudo_input"
      />
      <button @click="addUsername()">Send</button>
    </div>
    <span v-if="new_username_err" style="color: red"
      >Une erreur c'est produite : {{ new_username_err_msg }}</span
    >
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Session } from "../store/session";

const userModule = namespace("user");
const sessionModule = namespace("session");

@Component
export default class User extends Vue {
  @userModule.Action setUsername!: (payload: string) => void;
  @userModule.Action setUserID!: (payload: string) => void;
  @sessionModule.Getter session!: Session;
  new_username = "";
  new_username_err = false;
  new_username_err_msg = "";

  async addUsername() {
    const res = await fetch("http://localhost:3000/auction/register_user", {
      method: "PUT",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        auction_id: this.session.id,
        username: this.new_username,
      }),
    });
    if (res.status === 200) {
      this.new_username_err = false;
      this.new_username_err_msg = "";
      const body = await res.json();
      this.setUserID(body.user_id);
      this.setUsername(this.new_username);
    } else {
      this.new_username_err = true;
      this.new_username_err_msg = await res.text();
    }
  }
}
</script>

<style scoped>
.user_add_pseudo {
  display: flex;
  flex-direction: column;
  margin: auto;
}
</style>
