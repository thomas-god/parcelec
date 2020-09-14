<template>
  <div class="user_add_pseudo">
    <label for="user_add_pseudo_input">
      <h2>Choisissez un pseudo</h2>
    </label>

    <input
      type="text"
      v-model="new_username"
      v-on:keyup.enter="addUsername()"
      id="user_add_pseudo_input"
    />
    <!-- <button @click="addUsername()">▶️</button> -->

    <span v-if="new_username_err" style="color: red">{{
      new_username_err_msg
    }}</span>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Auction } from "../store/auction";

const userModule = namespace("user");
const auctionModule = namespace("auction");

@Component
export default class User extends Vue {
  @userModule.Action setUsername!: (payload: string) => void;
  @userModule.Action setUserID!: (payload: string) => void;
  @auctionModule.Getter auction!: Auction;
  @State("api_url") api_url!: string;
  new_username = "";
  new_username_err = false;
  new_username_err_msg = "";

  async addUsername() {
    if (this.new_username !== "") {
      const res = await fetch(
        `${this.api_url}/auction/${this.auction.id}/register_user`,
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
        this.setUsername(this.new_username);
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
</style>
