<script lang="ts">
  import type { StackSnapshot } from "$lib/message";
  import Battery from "../molecules/Battery.svelte";
  import Plant from "../molecules/Plant.svelte";
  import { sortStack } from "$lib/sortStack";

  let { plants, send }: { plants: StackSnapshot; send: (msg: string) => void } =
    $props();
  let sortedPlants = $derived(sortStack(plants));
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

<div class="flex flex-col gap-4 items-start">
  <!-- <h2 class="text-lg font-bold self-stretch pl-2">Centrales et clients</h2> -->
  {#each sortedPlants.entries() as [id, plant] (id)}
    <div class="px-2 mx-auto w-full">
      {#if plant.type === "Battery"}
        <Battery
          charge={plant.charge}
          setpoint={plant.output.setpoint}
          max_charge={plant.max_charge}
          updateSetpoint={(setpoint) => programSetpoint(id, setpoint)}
        />
      {:else if plant.type === "GasPlant"}
        <Plant
          cost={plant.output.cost}
          dispatchable={true}
          energy_cost={plant.settings.energy_cost}
          max_setpoint={plant.settings.max_setpoint}
          type="gaz"
          setpoint={plant.output.setpoint}
          updateSetpoint={(setpoint) => programSetpoint(id, setpoint)}
        />
      {:else if plant.type === "RenewablePlant"}
        <Plant
          cost={plant.output.cost}
          dispatchable={false}
          energy_cost={0}
          max_setpoint={plant.max_power}
          type="renewable"
          setpoint={plant.output.setpoint}
          updateSetpoint={(setpoint) => programSetpoint(id, setpoint)}
        />
      {:else if plant.type === "Consumers"}
        <Plant
          cost={plant.output.cost}
          dispatchable={false}
          energy_cost={0}
          max_setpoint={plant.max_power}
          type="consumers"
          setpoint={plant.output.setpoint}
          updateSetpoint={(setpoint) => programSetpoint(id, setpoint)}
        />
      {/if}
    </div>
  {/each}
</div>
