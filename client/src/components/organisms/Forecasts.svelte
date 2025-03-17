<script lang="ts">
  import type { StackForecasts, StackSnapshot } from "$lib/message";
  import PlantForecasts from "../molecules/PlantForecasts.svelte";

  let {
    plant_forecasts,
    plant_snapshots,
  }: {
    plant_forecasts: StackForecasts;
    plant_snapshots: StackSnapshot;
  } = $props();
</script>

<div class="flex flex-col overflow-y-auto gap-3">
  <PlantForecasts forecasts={plant_forecasts} plants={plant_snapshots} />
</div>
