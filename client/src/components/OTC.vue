<template> 
<div>
    <h2>Contrats directs</h2>
    <h3>Proposer un contrat</h3>
    <OTCAddNew/>
    <OTCList/>
</div>
</template>

<script lang='ts'>
import { Vue, Component } from "vue-property-decorator";
import OTCAddNew from "./OTCAddNew.vue";
import OTCList from "./OTCList.vue";

@Component({ components: { OTCAddNew, OTCList } })
export default class OTC extends Vue {}
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