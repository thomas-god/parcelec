<template>
  <div>
    <OTCItem v-for="otc in otcs" :key="otc.id" :item="otc"/>
  </div>
</template>

<script lang='ts'>
import { Vue, Component } from "vue-property-decorator";
import { namespace } from "vuex-class";
import { OTC } from "../store/otc";
import OTCItem from "./OTCItem.vue";

const otc_module = namespace("otcs");

@Component({ components: { OTCItem } })
export default class OTCList extends Vue {
  @otc_module.State otcs!: OTC[];
}
</script>

<style>
</style>