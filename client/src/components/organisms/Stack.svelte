<script lang="ts">
  import type { StackSnapshot } from "$lib/message";
  import Battery from "../molecules/Battery.svelte";
  import GenericPlant from "../molecules/GenericPlant.svelte";
  import { sortStack } from "$lib/sortStack";
  import NuclearPlant from "../molecules/NuclearPlant.svelte";

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

  let consumers = $derived.by(() => {
    let setpoint = 0;
    for (const [_, plant] of plants.entries()) {
      if (plant.type === "Consumers") {
        setpoint += plant.output.setpoint;
      }
    }
    return setpoint;
  });

  let renewable = $derived.by(() => {
    let setpoint = 0;
    for (const [_, plant] of plants.entries()) {
      if (plant.type === "RenewablePlant") {
        setpoint += plant.output.setpoint;
      }
    }
    return setpoint;
  });
</script>

<div class="flex flex-col gap-4 pt-4 items-start">
  <div class="flex flex-row justify-between w-full pl-1 pr-2">
    <div>
      <span class="text-2xl">🏙️</span> <span class="italic">Clients :</span>
      {consumers.toLocaleString("fr-FR", {
        signDisplay: "exceptZero",
      })} MW
    </div>
    <div>
      <span class="text-2xl">☀️️</span>
      <span class="italic">Solaire</span> : {renewable.toLocaleString("fr-FR", {
        signDisplay: "exceptZero",
      })} MW
    </div>
  </div>
  {#each sortedPlants.entries() as [id, plant] (id)}
    {#if plant.type === "Battery"}
      <div class="pl-1 pr-2 mx-auto w-full">
        <Battery
          charge={plant.charge}
          setpoint={plant.output.setpoint}
          max_charge={plant.max_charge}
          updateSetpoint={(setpoint) => programSetpoint(id, setpoint)}
        />
      </div>
    {:else if plant.type === "GasPlant"}
      <div class="pl-1 pr-2 mx-auto w-full">
        <GenericPlant
          cost={plant.output.cost}
          dispatchable={true}
          energy_cost={plant.settings.energy_cost}
          max_setpoint={plant.settings.max_setpoint}
          type="gaz"
          setpoint={plant.output.setpoint}
          updateSetpoint={(setpoint) => programSetpoint(id, setpoint)}
        />
      </div>
      <!-- {:else if plant.type === "RenewablePlant"}
        <GenericPlant
          cost={plant.output.cost}
          dispatchable={false}
          energy_cost={0}
          max_setpoint={plant.max_power}
          type="renewable"
          setpoint={plant.output.setpoint}
          updateSetpoint={(setpoint) => programSetpoint(id, setpoint)}
        /> -->
      <!-- {:else if plant.type === "Consumers"}
        <GenericPlant
          cost={plant.output.cost}
          dispatchable={false}
          energy_cost={0}
          max_setpoint={plant.max_power}
          type="consumers"
          setpoint={plant.output.setpoint}
          updateSetpoint={(setpoint) => programSetpoint(id, setpoint)}
        /> -->
    {:else if plant.type === "Nuclear"}
      <div class="pl-1 pr-2 mx-auto w-full">
        <NuclearPlant
          cost={plant.output.cost}
          dispatchable={!plant.locked}
          setpoint={plant.output.setpoint}
          previous_setpoint={plant.previous_setpoint}
          max_setpoint={plant.max_setpoint}
          energy_cost={plant.energy_cost}
          updateSetpoint={(setpoint) => programSetpoint(id, setpoint)}
        />
      </div>
    {/if}
  {/each}
</div>
