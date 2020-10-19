<template>
  <div>
    <h2>Contrats directs</h2>
    <h3 v-if="can_post_planning">Proposer un contrat</h3>
    <OTCAddNew v-if="can_post_planning" />
    <OTCList />
  </div>
</template>

<script lang='ts'>
import { Vue, Component } from "vue-property-decorator";
import { namespace } from "vuex-class";
import OTCAddNew from "./OTCAddNew.vue";
import OTCList from "./OTCList.vue";

const session_module = namespace("session");

@Component({ components: { OTCAddNew, OTCList } })
export default class OTC extends Vue {
  @session_module.Getter can_post_planning!: boolean;
}
</script>

<style scoped>
h2 {
  margin-top: 0;
}
h3 {
  margin-left: 0;
  padding-left: 2rem;
  text-align: start;
}
</style>