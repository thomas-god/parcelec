<script lang="ts">
  import type { GasPlantState } from "$lib/message";

  let { plant }: { plant: GasPlantState } = $props();
  let setpoint_percentage = $derived(
    (plant.setpoint / plant.settings.max_setpoint) * 100,
  );
</script>

<div class="flex flex-col @container max-w-[400px]">
  <!-- <div>🔋</div> -->
  <div class="mb-5 h-6 rounded-full bg-gray-200 grow grid grid-rows-1">
    <div
      class="h-6 rounded-full bg-orange-500 col-start-1 col-end-2"
      style="width: {setpoint_percentage}%;"
    ></div>
  </div>
  <div class="flex flex-col @sm:flex-row @sm:justify-between">
    <div>
      Consigne: {plant.setpoint} / {plant.settings.max_setpoint} MW
    </div>
    <div>Coût: {plant.cost}€ ({plant.settings.energy_cost} €/MWh)</div>
  </div>
</div>
