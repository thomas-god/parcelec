<template>
  <div :style="containerStyle">
    <ChatroomMessages class="chatroom__box chatroom__messages" />
    <ChatroomUsersList
      class="chatroom__box chatroom__users_list"
      :display_ready_status="display_ready"
    />
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import ChatroomMessages from "./ChatroomMessages.vue";
import ChatroomUsersList from "./ChatroomUsersList.vue";

@Component({ components: { ChatroomMessages, ChatroomUsersList } })
export default class Chatroom extends Vue {
  @Prop({ default: false }) display_ready!: boolean;
  @Prop({ default: "row" }) display_direction!: "row" | "column";

  get containerStyle(): string {
    return `
      display: flex;
      flex-direction: ${this.display_direction};
    `;
  }
}
</script>

<style scoped>
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
