<script lang="ts">
  import type { GasPlantState } from "$lib/message";

  let {
    plant,
    updateSetpoint,
  }: { plant: GasPlantState; updateSetpoint: (setpoint: number) => void } =
    $props();
  let setpoint_percentage = $derived(
    (plant.output.setpoint / plant.settings.max_setpoint) * 100,
  );
  let current_setpoint = $state("0");
  $effect(() => {
    current_setpoint = String(plant.output.setpoint);
  });

  let debounceTimer: ReturnType<typeof setTimeout>;
  const deboundedUpdateSetpoint = () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      updateSetpoint(Number.parseInt(current_setpoint));
    }, 500);
  };
</script>

<div
  class="flex flex-col @container max-w-[400px] self-center"
  style="width: 100%;"
>
  <div class="flex flex-row">
    <div>🔥</div>
    <div class="mb-1 h-6 rounded-full bg-gray-200 grow grid grid-rows-1">
      <div
        class="h-6 rounded-full bg-orange-500 col-start-1 col-end-2"
        style="width: {setpoint_percentage}%;"
      ></div>
    </div>
  </div>
  <div class="flex flex-col @xs:flex-row @xs:justify-between">
    <div>
      <label
        >Consigne
        <input
          type="text"
          inputmode="numeric"
          pattern="[0-9]*"
          bind:value={current_setpoint}
          oninput={deboundedUpdateSetpoint}
          class="max-w-[60px] text-center"
          step="10"
        />
        MW
      </label>
    </div>
    <div>
      {(-plant.output.cost).toLocaleString("fr-FR")}€ ({plant.settings
        .energy_cost} €/MWh)
    </div>
  </div>
</div>
