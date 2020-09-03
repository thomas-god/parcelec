<template>
  <div>
    <user v-if="!username" />
    <div v-else>
      <p>Hello {{ username }} !</p>
      <messages />
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { UserState } from "../store/user";
import User from "./User.vue";
import Messages from "./Messages.vue";

const userModule = namespace("user");

@Component({
  components: { User, Messages },
})
export default class HelloWorld extends Vue {
  @Prop() private msg!: string;
  @userModule.Getter username!: string;
}
</script>
