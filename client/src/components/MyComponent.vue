<template>
  <div>
    <SessionSelect v-if="!session.id" />
    <UsernameSelect v-if="session.id && !username" />
    <div v-if="session.id && username">
      <p>Hello {{ username }} !</p>
      <SessionSalon />
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Session } from "../store/session";
import SessionSelect from "./SessionSelect.vue";
import UsernameSelect from "./UsernameSelect.vue";
import SessionSalon from "./SessionSalon.vue";
import Messages from "./Messages.vue";

const userModule = namespace("user");
const sessionModule = namespace("session");

@Component({
  components: { SessionSelect, UsernameSelect, SessionSalon },
})
export default class HelloWorld extends Vue {
  @userModule.Getter username!: string;
  @sessionModule.Getter session!: Session;
}
</script>
