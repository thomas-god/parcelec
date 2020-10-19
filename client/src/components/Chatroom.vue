<template>
  <div class="chatroom__container">
    <h2>Messages</h2>
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
}
</script>

<style scoped>
h2 {
  margin-top: 0;
}
@media screen and (min-width: 750px) {
  .chatroom__container {
    display: flex;
    flex-direction: column;
    flex-wrap: nowrap;
  }

  .chatroom__messages {
    flex-grow: 2;
    flex-basis: 0;
  }

  .chatroom__users_list {
    flex-grow: 1;
    flex-basis: 0;
  }
}

@media screen and (max-width: 750px) {
  .chatroom__container {
    display: flex;
    flex-direction: column;
    flex-wrap: nowrap;
  }
}

</style>
