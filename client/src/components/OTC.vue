<template>
  <div>
    <h2>Contrats directs</h2>
    <h3 v-if="dummy || can_post_planning">Proposer un contrat</h3>
    <OTCAddNew v-if="dummy || can_post_planning" :dummy="dummy" />
    <OTCList />
  </div>
</template>

<script lang='ts'>
import { Vue, Component, Prop } from "vue-property-decorator";
import { namespace } from "vuex-class";
import OTCAddNew from "./OTCAddNew.vue";
import OTCList from "./OTCList.vue";

const session_module = namespace("session");

@Component({ components: { OTCAddNew, OTCList } })
export default class OTC extends Vue {
  @Prop({ default: false }) dummy!: boolean;
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