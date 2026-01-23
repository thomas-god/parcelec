<script lang="ts">
  import type { Plant, StackSnapshot } from "$lib/message";
  import Battery from "../molecules/Battery.svelte";
  import GasPlant from "../molecules/GasPlant.svelte";
  import NuclearPlant from "../molecules/NuclearPlant.svelte";

  let { plants, send }: { plants: StackSnapshot; send: (msg: string) => void } =
    $props();
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

  let other_plants = $derived.by(() => {
    const other_plants = new Map<string, Plant>();
    /// Cannot use plants.entries().find(/.../) on WebKit...
    const _plants = [...plants.entries()];

    let gas_plant = _plants.find(([_, plant]) => plant.type === "GasPlant");
    if (gas_plant !== undefined) {
      other_plants.set(gas_plant[0], gas_plant[1]);
    }
    let nuke_plant = _plants.find(([_, plant]) => plant.type === "Nuclear");
    if (nuke_plant !== undefined) {
      other_plants.set(nuke_plant[0], nuke_plant[1]);
    }
    let battery = _plants.find(([_, plant]) => plant.type === "Battery");
    if (battery !== undefined) {
      other_plants.set(battery[0], battery[1]);
    }

    return other_plants;
  });
</script>

<div class="flex flex-col gap-4 pt-4 items-start">
  <div class="flex flex-row justify-between w-full pl-1 pr-2">
    <div class="flex flex-row gap-2">
      <img
        src="/icons/consumers.svg"
        alt="Consumers city icon"
        class="w-8 h-8"
      />
      <span>
        <span class="italic">Clients :</span>
        {consumers.toLocaleString("fr-FR", {
          signDisplay: "exceptZero",
        })} MW
      </span>
    </div>
    <div class="flex flex-row gap-2">
      <img
        src="/icons/renewable.svg"
        alt="Renewable plant icon"
        class="w-8 h-8"
      />
      <span>
        <span class="italic">Solaire</span> : {renewable.toLocaleString(
          "fr-FR",
          {
            signDisplay: "exceptZero",
          },
        )} MW
      </span>
    </div>
  </div>
  {#each other_plants.entries() as [id, plant] (id)}
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
        <GasPlant
          cost={plant.output.cost}
          dispatchable={true}
          energy_cost={plant.settings.energy_cost}
          max_setpoint={plant.settings.max_setpoint}
          setpoint={plant.output.setpoint}
          updateSetpoint={(setpoint) => programSetpoint(id, setpoint)}
        />
      </div>
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
