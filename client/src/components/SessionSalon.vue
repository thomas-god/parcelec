<template>
  <div>
    <ul class="message_messages">
      <li v-for="msg in msgs" :key="msg.date" class="message_msg">
        <span class="message_msg_user">{{ msg.username }} :</span>
        <span class="message_msg_text">{{ msg.data }} </span>
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
          autofocus
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
const sessionModule = namespace("session");

interface ClientMessage {
  username: string;
  data: string;
  date: string;
}

@Component
export default class Messages extends Vue {
  private socket!: WebSocket;
  @userModule.Getter username!: string;
  @userModule.Getter user_id!: string;
  @sessionModule.Getter session_id!: string;

  openWebSocket(): void {
    // Create WebSocket connection.
    this.socket = new WebSocket("ws://localhost:3000");

    this.socket.addEventListener("open", () => {
      this.socket.send(this.getPayload("handshake", ""));
    });

    this.socket.addEventListener("close", (event) => {
      console.log("closing", event.reason);
    });

    // Listen for messages
    this.socket.addEventListener("message", (event) => {
      try {
        const message = JSON.parse(event.data);
        if (message.reason === "message") {
          this.addMsg(message);
        }
      } catch (error) {
        console.log(error);
      }
    });
  }

  /**
   * Helper function to get the correct payload for WebSocket exchanges
   */
  getPayload(reason: string, data: any) {
    return JSON.stringify({
      username: this.username,
      reason: reason,
      credentials: {
        session_id: this.session_id,
        user_id: this.user_id,
      },
      data: data,
    });
  }

  created(): void {
    this.openWebSocket();
  }

  // Messages from chatroom
  msgs: ClientMessage[] = [];
  addMsg(msg: ClientMessage): void {
    const new_msg = { username: msg.username, date: msg.date, data: msg.data };
    this.msgs.push(new_msg);
  }

  // Post new message
  new_msg = "";
  sendMsg(): void {
    if (this.new_msg) {
      this.socket.send(this.getPayload("message", this.new_msg));
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
  margin: 1rem auto;
  max-width: 500px;
  height: 300px;
  border: 1px solid black;
  border-radius: 3px;
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
