<template>
  <div>
    <ul class="message__messages">
      <li v-for="msg in messages" :key="msg.date" class="message__msg">
        <span class="message__msg_user">{{ msg.username }} :</span>
        <span class="message__msg_text">{{ msg.data }} </span>
        <span class="message__msg_hour">{{ getHourFromDate(msg.date) }} </span>
      </li>
    </ul>
    <div class="message__input">
      <!-- <label for="message_add_msg_input">Entrez votre message</label> -->
      <input
        type="text"
        id="message_add_msg_input"
        v-model="new_msg"
        v-on:keyup.enter="postMsg()"
        autofocus
      />
      <Btn @click="postMsg()" background_color="rgba(0,0,0,0)">▶️</Btn>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { ClientMessage } from "../store/webSocket";
import Btn from "./base/Button.vue";

const webSocketModule = namespace("webSocket");

@Component({ components: { Btn } })
export default class Messages extends Vue {
  @webSocketModule.Action sendMsg!: (payload: string) => void;
  @webSocketModule.State messages!: ClientMessage[];

  new_msg = "";
  /**
   * Post a new message via websocket
   */
  postMsg(): void {
    if (this.new_msg) {
      this.sendMsg(this.new_msg);
      this.new_msg = "";
    }
  }

  /**
   * Format the date
   */
  getHourFromDate(date_string: string): string {
    const date = new Date(date_string);
    return `${String(date.getHours()).padStart(2, "0")}:${String(
      date.getMinutes()
    ).padStart(2, "0")}`;
  }
}
</script>

<style scoped>
h3 {
  margin: 0;
}
.message__input {
  width: 90%;
  margin: auto;
  display: flex;
  flex-direction: row;
  align-items: center;
}

.message__input input {
  font-size: 1rem;
  width: 90%;
}

.message__messages {
  padding: 5px;
  margin: 1rem auto;
  height: 300px;
  border: 1px solid rgba(128, 128, 128, 0.596);
  overflow-y: scroll;
}

.message__msg {
  display: flex;
  flex-direction: row;
  flex-wrap: nowrap;
  justify-content: flex-start;
  align-items: stretch;
  align-content: stretch;
}

.message__msg .message__msg_user {
  font-weight: bold;
  flex-grow: 0;
  flex-shrink: 0;
  width: 75px;
  text-align: end;
  padding-right: 10px;
}
.message__msg .message__msg_text {
  flex-grow: 1;
  text-align: start;
}
.message__msg .message__msg_hour {
  padding-left: 10px;
  flex-grow: 0;
  flex-shrink: 0;
  font-weight: lighter;
  font-style: italic;
}
</style>
