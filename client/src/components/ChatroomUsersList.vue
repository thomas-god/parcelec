<template>
  <div>
    <h3>Utilisateurs</h3>
    <ul>
      <li v-for="user in users" :key="user">
        {{ user }}
      </li>
    </ul>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";

const auctionModule = namespace("auction");

@Component
export default class UserList extends Vue {
  @auctionModule.State users!: string[];
}
</script>

<style scoped>
h3 {
  margin-top: 0;
  margin-bottom: 0;
}
ul {
  margin-bottom: 0;
}
li {
  text-align: start !important;
}
</style>
