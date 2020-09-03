<template>
  <div>
    <ul class="message_messages">
      <li v-for="msg in msgs" :key="msg.date" class="message_msg">
        <span class="message_msg_user">{{ msg.username }} :</span>
        <span class="message_msg_text">{{ msg.msg }} </span>
        <span class="message_msg_hour">{{ getHourFromDate(msg.date) }} </span>
      </li>
    </ul>
    <div class="message_input">
      <label for="message_add_msg_input">Entrez votre message</label>
      <div>
        <input
          type="text"
          id="message_add_msg_input"
          v-model="new_msg"
          v-on:keyup.enter="sendMsg()"
        />
        <button @click="sendMsg()">Send</button>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";

const userModule = namespace("user");

interface ClientMessage {
  username: string;
  message: string;
  date: Date;
}

@Component
export default class Messages extends Vue {
  private socket!: WebSocket;

  async created(): Promise<void> {
    // Create WebSocket connection.
    this.socket = new WebSocket("ws://localhost:3000");

    // Listen for messages
    this.socket.addEventListener("message", (event) => {
      this.addMsg(event.data);
    });
  }

  // Messages from chatroom
  msgs: ClientMessage[] = [];
  addMsg(msg: string): void {
    this.msgs.push(JSON.parse(msg) as ClientMessage);
  }

  // Post new message
  @userModule.Getter username!: string;
  new_msg = "";
  sendMsg(): void {
    if (this.new_msg) {
      this.socket.send(
        JSON.stringify({ username: this.username, msg: this.new_msg })
      );
      this.new_msg = "";
    }
  }

  getHourFromDate(date_string: string): string {
    const date = new Date(date_string);
    return `${String(date.getHours()).padStart(2, "0")}:${String(
      date.getMinutes()
    ).padStart(2, "0")}`;
  }
}
</script>

<style scoped>
.message_input {
  max-width: 300px;
  margin: auto;
  display: flex;
  flex-direction: column;
}

.message_messages {
  padding: 0;
  margin: auto;
  max-width: 500px;
}

.message_msg {
  display: flex;
  flex-direction: row;
  flex-wrap: nowrap;
  justify-content: flex-start;
  align-items: stretch;
  align-content: stretch;
}

.message_msg .message_msg_user {
  font-weight: bold;
  flex-grow: 0;
  flex-shrink: 0;
  width: 75px;
  text-align: end;
  padding-right: 10px;
}
.message_msg .message_msg_text {
  flex-grow: 1;
  text-align: start;
}
.message_msg .message_msg_hour {
  padding-left: 10px;
  flex-grow: 0;
  flex-shrink: 0;
  font-weight: lighter;
  font-style: italic;
}
</style>
