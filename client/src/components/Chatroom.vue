<template>
  <div class="chatroom__container">
    <ChatroomUsersList class="chatroom__box" />
    <ChatroomMessages class="chatroom__box" />
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
  flex-direction: column;
}

.chatroom__box {
  border: 1px solid black;
  margin: 1rem;
  padding: 1rem;
  max-width: 450px;
}
</style>
