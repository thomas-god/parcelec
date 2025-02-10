<script lang="ts">
  import type { GasPlant } from "$lib/message";

  let { plant }: { plant: GasPlant } = $props();
</script>

{JSON.stringify(plant)}
