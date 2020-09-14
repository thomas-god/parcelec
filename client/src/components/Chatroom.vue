<template>
  <div class="chatroom__container">
    <ChatroomMessages class="chatroom__box chatroom__messages" />
    <ChatroomUsersList
      class="chatroom__box chatroom__users_list"
      :display_ready_status="true"
    />
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import ChatroomMessages from "./ChatroomMessages.vue";
import ChatroomUsersList from "./ChatroomUsersList.vue";

@Component({ components: { ChatroomMessages, ChatroomUsersList } })
export default class Chatroom extends Vue {}
</script>

<style scoped>
.chatroom__container {
  display: flex;
  flex-direction: row;
}

.chatroom__box {
  border: 1px solid black;
  margin: 1rem;
  padding: 1rem;
}

.chatroom__messages {
  flex-grow: 4;
}

.chatroom__users_list {
  flex-grow: 1;
}
</style>
