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

<h2 class="font-semibold text-lg pl-4">Centrales ⚡</h2>

<div class="tutorial-container">
  <p class="pt-2 tutorial-left">
    Vous disposez de centrales que vous pouvez piloter pour vous équilibrer.
    Essayez de réduire votre déficit avec votre centrale à gaz !
  </p>

  <div class="my-1 mx-auto w-full tutorial-right">
    {#each plants as [id, plant] (id)}
      {#if plant.type === "GasPlant"}
        <div>
          <GasPlant
            cost={plant.output.cost}
            dispatchable={true}
            energy_cost={plant.settings.energy_cost}
            max_setpoint={plant.settings.max_setpoint}
            setpoint={plant.output.setpoint}
            updateSetpoint={(setpoint) => programSetpoint(id, setpoint)}
          />
        </div>
      {/if}
    {/each}
  </div>

  <div class="tutorial-left pt-2">
    <p>Il existe plusieurs types de centrales :</p>
    <ul>
      <li>
        🔥 <i>centrale à gaz</i> : entièrement pilotable mais coûteuse à exploiter,
      </li>
      <li>
        ☢️ <i>centrale nucléaire</i> : peu chère, mais changer sa production la bloquera
        pour la période suivante,
      </li>
      <li>
        ☀️ <i>centrale solaire</i> : ne coûte rien mais a une production variable
        que vous ne pouvez contrôler,
      </li>
      <li>
        🔋 <i>batterie</i> : vous pouvez choisir de la charger ou décharger pour stocker
        de l'énergie d'une période à l'autre.
      </li>
    </ul>
  </div>
</div>

<style>
  @media (width > 800px) {
    .tutorial-container {
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      column-gap: 2rem;
      align-items: center;
    }

    .tutorial-left {
      grid-column: 1 /2;
    }

    .tutorial-right {
      grid-column: 2/3;
      grid-row: 1 / 3;
    }

    .tutorial-right > div {
      background-color: var(--color-base-100);
      border-radius: var(--radius-lg);
      padding: calc(var(--spacing) * 5) calc(var(--spacing) * 1.5);
      display: flex;
      flex-direction: row;
    }
  }
</style>
