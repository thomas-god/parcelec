<script lang="ts">
  import GasPlant from "../GasPlant.svelte";
  import type { StackSnapshot } from "$lib/message";

  let {
    sendMessage,
    plants,
  }: { sendMessage: (msg: string) => void; plants: StackSnapshot } = $props();

  const programSetpoint = (plant_id: string, setpoint: number) => {
    const parsed_setpoint = Number.isNaN(setpoint) ? 0 : setpoint;
    sendMessage(
      JSON.stringify({
        ProgramPlant: {
          plant_id,
          setpoint: parsed_setpoint,
        },
      }),
    );
  };
</script>

<h2 class="font-semibold text-lg max-[500px]:pl-4">Centrales ⚡</h2>

<p class="px-4 pt-2">
  Vous disposez de centrales que vous pouvez piloter pour vous équilibrer.
  Essayez de réduire votre déficit avec votre centrale à gaz !
</p>

<div class="p-5">
  {#each plants as [id, plant] (id)}
    {#if plant.type === "GasPlant"}
      <GasPlant
        cost={plant.output.cost}
        dispatchable={true}
        energy_cost={plant.settings.energy_cost}
        max_setpoint={plant.settings.max_setpoint}
        setpoint={plant.output.setpoint}
        updateSetpoint={(setpoint) => programSetpoint(id, setpoint)}
      />
    {/if}
  {/each}
</div>

<p class="px-4">Il existe plusieurs types de centrales :</p>
<ul class="px-4">
  <li>
    🔥 <i>centrale à gaz</i> : entièrement pilotable mais coûteuse à exploiter,
  </li>
  <li>
    ☢️ <i>centrale nucléaire</i> : peu chère, mais changer sa production la bloquera
    pour la période suivante,
  </li>
  <li>
    ☀️ <i>centrale solaire</i> : ne coûte rien mais a une production variable que
    vous ne pouvez contrôler,
  </li>
  <li>
    🔋 <i>batterie</i> : vous pouvez choisir de la charger ou décharger pour stocker
    de l'énergie d'une période à l'autre.
  </li>
</ul>
