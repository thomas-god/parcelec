<template>
  <div>
    <p>Choisissez un pseudo dans la liste :</p>
    <ul class="usernames_list">
      <li
        v-for="name in usernames_available"
        :key="name"
        @click="setUsername(name)"
      >
        <span>{{ name }}</span>
      </li>
    </ul>
    <div class="add_pseudo">
      <label for="user_add_pseudo_input">
        Ou bien entrez un nouveau pseudo :
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
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
const userModule = namespace("user");

@Component
export default class User extends Vue {
  @userModule.Getter username!: string;
  @userModule.Action setUsername!: (payload: string) => void;

  // Usernames available
  usernames_available: string[] = [];
  async getAvailableUsernames(): Promise<void> {
    const res = await fetch("http://localhost:3000/user/list", {
      method: "GET",
    });
    this.usernames_available = await res.json();
  }

  // Choose a custom username
  new_username = "";
  new_username_err = false;
  new_username_err_msg = "";
  async addUsername() {
    const res = await fetch("http://localhost:3000/user/new", {
      method: "PUT",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ username: this.new_username }),
    });
    if (res.status === 200) {
      this.new_username_err = false;
      this.new_username_err_msg = "";
      this.setUsername(this.new_username);
    } else {
      this.new_username_err = true;
      this.new_username_err_msg = await res.text();
    }
  }

  async created(): Promise<void> {
    await this.getAvailableUsernames();
  }
}
</script>

<style scoped>
.usernames_list {
  max-width: 200px;
  margin: auto;
  margin-bottom: 1.5rem;
  padding: 1rem;
  border: 1px solid rgba(0, 0, 0, 0.493);
  border-radius: 3px;
  box-shadow: 12px 12px 2px 1px rgba(28, 28, 56, 0.26);
}

.add_user_pseudo {
  display: flex;
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
</style>
