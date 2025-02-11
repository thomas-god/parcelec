<script lang="ts">
  import type { RenewablePlantState } from "$lib/message";

  let {
    plant,
  }: {
    plant: RenewablePlantState;
  } = $props();
  let setpoint_percentage = $derived((plant.setpoint / plant.max_power) * 100);
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
      Production: {plant.setpoint} MW
    </div>
    <div>Coût: 0€</div>
  </div>
</div>
