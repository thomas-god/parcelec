<template>
  <div>
    <h3 v-if="otcs_send.length > 0">Contrats envoyés</h3>
    <OTCItem
      v-for="otc in otcs_send"
      :key="otc.id"
      :item="otc"
      :type="'send'"
    />
    <h3 v-if="otcs_received.length > 0">Contrats reçus</h3>
    <OTCItem
      v-for="otc in otcs_received"
      :key="otc.id"
      :item="otc"
      :type="'received'"
    />
    <h3 v-if="otcs.length === 0">Vous n'avez pas de contrats</h3>
  </div>
</template>

<script lang='ts'>
import { Vue, Component } from "vue-property-decorator";
import { namespace } from "vuex-class";
import { OTC } from "../store/otc";
import OTCItem from "./OTCItem.vue";

const user_module = namespace("user");
const otc_module = namespace("otcs");

@Component({ components: { OTCItem } })
export default class OTCList extends Vue {
  @user_module.Getter username!: string;
  @otc_module.State otcs!: OTC[];

  get otcs_send(): OTC[] {
    return this.otcs
      .filter(otc => otc.user_from === this.username)
      .sort((a, b) => (a.status > b.status ? 1 : -1));
  }

  get otcs_received(): OTC[] {
    return this.otcs
      .filter(otc => otc.user_to === this.username)
      .sort((a, b) => (a.status > b.status ? 1 : -1));
  }
}
</script>

<style scoped>
h3 {
  margin-left: 0;
  padding-left: 2rem;
  text-align: start;
}
</style>