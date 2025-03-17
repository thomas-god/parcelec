<script lang="ts">
  import type {
    MarketForecast,
    StackForecasts,
    StackSnapshot,
  } from "$lib/message";
  import MarketForecasts from "../molecules/MarketForecasts.svelte";
  import PlantForecasts from "../molecules/PlantForecasts.svelte";

  let {
    plant_forecasts,
    plant_snapshots,
    market_forecasts,
  }: {
    plant_forecasts: StackForecasts;
    plant_snapshots: StackSnapshot;
    market_forecasts: MarketForecast[];
  } = $props();
</script>

<div class="flex flex-col overflow-y-auto gap-3">
  <div>
    <PlantForecasts forecasts={plant_forecasts} plants={plant_snapshots} />
  </div>
  {#if false}
    <MarketForecasts {market_forecasts} />
  {/if}
</div>
