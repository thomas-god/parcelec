<script lang="ts">
  import type { ConsumersState } from "$lib/message";

  let {
    plant,
  }: {
    plant: ConsumersState;
  } = $props();
  let setpoint_percentage = $derived(
    Math.abs((plant.output.setpoint / plant.max_power) * 100),
  );
</script>

<div
  class="flex flex-col @container max-w-[400px] self-center"
  style="width: 100%;"
>
  <div class="flex flex-row">
    <div>🏙️</div>
    <div class="mb-1 h-6 rounded-full bg-gray-200 grow grid grid-rows-1">
      <div
        class="h-6 rounded-full bg-orange-500"
        style="width: {setpoint_percentage}%;"
      ></div>
    </div>
  </div>
  <div class="flex flex-row justify-between">
    <div>
      Consommation: {plant.output.setpoint} MW
    </div>
    <div>{(-plant.output.cost).toLocaleString("fr-FR")}€</div>
  </div>
</div>
