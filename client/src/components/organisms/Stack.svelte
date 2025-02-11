<script lang="ts">
  import type { StackSnapshot } from "$lib/message";
  import Battery from "../molecules/Battery.svelte";
  import GasPlant from "../molecules/GasPlant.svelte";
  import RenewablePlant from "../molecules/RenewablePlant.svelte";

  let { plants, send }: { plants: StackSnapshot; send: (msg: string) => void } =
    $props();
  let sortedPlant = $derived(new Map([...plants.entries()].sort()));
  const programSetpoint = (plant_id: string, setpoint: number) => {
    send(
      JSON.stringify({
        ProgramPlant: {
          plant_id,
          setpoint,
        },
      }),
    );
  };
</script>

<div class="flex flex-col gap-4 items-center">
  <h2 class="text-lg font-bold self-stretch">Centrales</h2>
  {#each sortedPlant.entries() as [id, plant] (id)}
    {#if plant.type === "Battery"}
      <Battery
        battery={plant}
        updateSetpoint={(setpoint) => programSetpoint(id, setpoint)}
      />
    {:else if plant.type === "GasPlant"}
      <GasPlant
        {plant}
        updateSetpoint={(setpoint) => programSetpoint(id, setpoint)}
      />
    {:else if plant.type === "RenewablePlant"}
      <RenewablePlant {plant} />
    {/if}
  {/each}
</div>
