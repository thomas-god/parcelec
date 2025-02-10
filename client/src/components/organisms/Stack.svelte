<script lang="ts">
  import type { StackSnapshot } from "$lib/message";
  import Battery from "../molecules/Battery.svelte";
  import GasPlant from "../molecules/GasPlant.svelte";

  let { plants }: { plants: StackSnapshot } = $props();
</script>

{#each plants.entries() as [id, plant] (id)}
  {#if plant.type === "Battery"}
    <Battery battery={plant} />
  {:else if plant.type === "GasPlant"}
    <GasPlant {plant} />
  {/if}
{/each}
