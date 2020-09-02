<template>
  <div>
    <div v-if="!username">
      <p>Choisissez un nom</p>
      <ul>
        <li
          v-for="name in usernames_available"
          :key="name"
          @click="username = name"
        >
          {{ name }}
        </li>
      </ul>
      <label>
        Votre nom
        <input type="text" v-model="new_username" />
      </label>
      <button @click="addUsername()">Send</button>
    </div>
    <div v-else>
      <p>Hello {{ username }} !</p>
      <ul>
        <li v-for="msg in msgs" :key="msg.date">
          {{ msg.username }}: {{ msg.msg }}
        </li>
      </ul>
      <input type="text" v-model="new_msg" />
      <button @click="sendMsg()">Send</button>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { UserState } from "../store/user";

const user = namespace("user");

interface ClientMessage {
  username: string;
  message: string;
  date: Date;
}

@Component
export default class HelloWorld extends Vue {
  @Prop() private msg!: string;
  private socket!: WebSocket;

  username = "";
  // Username available
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
  async addUsername() {
    const res = await fetch("http://localhost:3000/user/new", {
      method: "PUT",
      body: JSON.stringify({ username: this.new_username }),
    });
    if (res.status === 200) {
      this.new_username_err = false;
      this.username = this.new_username;
    } else {
      this.new_username_err = true;
    }
  }

  async created(): Promise<void> {
    await this.getAvailableUsernames();
    // Create WebSocket connection.
    this.socket = new WebSocket("ws://localhost:3000");

    // Connection opened
    this.socket.addEventListener("open", (event) => {
      //(this.socket as WebSocket).send("Hello Server!");
    });

    // Listen for messages
    this.socket.addEventListener("message", (event) => {
      this.addMsg(event.data);
    });
  }

  // Message from chatroom
  msgs: ClientMessage[] = [];
  new_msg = "";
  addMsg(msg: string): void {
    this.msgs.push(JSON.parse(msg) as ClientMessage);
  }
  sendMsg(): void {
    this.socket.send(
      JSON.stringify({ username: this.username, msg: this.new_msg })
    );
  }
}
</script>

<style scoped>
li:hover {
  background-color: rgb(0, 151, 98);
}
</style>
