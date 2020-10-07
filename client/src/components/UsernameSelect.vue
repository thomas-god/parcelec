<template>
  <div class="user_add_pseudo">
    <label for="user_add_pseudo_input">
      <h1>Bienvenue sur Parcélec ! ⚡️</h1>
      <h2>Choisissez un pseudo</h2>
    </label>

    <input
      type="text"
      v-model="new_username"
      v-on:keyup.enter="addUsername()"
      id="user_add_pseudo_input"
    />
    <Btn @click="addUsername()">OK</Btn>

    <span v-if="new_username_err" style="color: red">{{
      new_username_err_msg
    }}</span>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Session } from "../store/session";
import Btn from './base/Button.vue'

const userModule = namespace("user");
const sessionModule = namespace("session");

@Component({ components: { Btn }})
export default class User extends Vue {
  // Store related
  @userModule.Action setUserID!: (payload: string) => void;
  @sessionModule.Getter session!: Session;
  @State("api_url") api_url!: string;

  // Username input stuff
  new_username = "";
  new_username_err = false;
  new_username_err_msg = "";

  /**
   * Tries to register a new user with the username
   * from the input. If username insertion succeeds, trigger
   * the loadGameContent action.
   */
  async addUsername() {
    if (this.new_username !== "") {
      const res = await fetch(
        `${this.api_url}/session/${this.session.id}/register_user`,
        {
          method: "PUT",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            username: this.new_username,
          }),
        }
      );
      if (res.status === 201) {
        this.new_username_err = false;
        this.new_username_err_msg = "";
        const body = await res.json();
        this.setUserID(body.user_id);
        this.$router.push(`/session/${this.session.id}/user/${body.user_id}`);
      } else {
        this.new_username_err = true;
        this.new_username_err_msg = await res.text();
      }
    } else {
      this.new_username_err = true;
      this.new_username_err_msg = "Le pseudo ne doit pas être vide";
    }
  }
}
</script>

<style scoped>
.user_add_pseudo {
  display: flex;
  flex-direction: column;
  margin: auto;
  align-items: center;
}

.user_add_pseudo input {
  font-size: 1.5rem;
  text-align: center;
  margin-bottom: 10px;
}

.user_add_pseudo button {
  margin-top: 0.7rem;
  font-size: 1rem;
}
</style>
