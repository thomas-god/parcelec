<script lang="ts">
  import type { StackSnapshot } from "$lib/message";
  import { match, Pattern } from "ts-pattern";
  import Battery from "../molecules/Battery.svelte";
  import Consumers from "../molecules/Consumers.svelte";
  import GasPlant from "../molecules/GasPlant.svelte";
  import RenewablePlant from "../molecules/RenewablePlant.svelte";
  import { sortStack } from "$lib/sortStack";

  let { plants, send }: { plants: StackSnapshot; send: (msg: string) => void } =
    $props();
  let sortedPlants = $derived(sortStack(plants));
  $inspect(Array.from(sortedPlants.keys()));
  const programSetpoint = (plant_id: string, setpoint: number) => {
    const parsed_setpoint = Number.isNaN(setpoint) ? 0 : setpoint;
    send(
      JSON.stringify({
        ProgramPlant: {
          plant_id,
          setpoint: parsed_setpoint,
        },
      }),
    );
  };
</script>

<div class="flex flex-col gap-4 items-center">
  <h2 class="text-lg font-bold self-stretch">Centrales et clients</h2>
  {#each sortedPlants.entries() as [id, plant] (id)}
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
    {:else if plant.type === "Consumers"}
      <Consumers {plant} />
    {/if}
  {/each}
</div>
