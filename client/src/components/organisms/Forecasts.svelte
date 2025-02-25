<script lang="ts">
  import type { MarketForecast, StackForecasts } from "$lib/message";

  let {
    plant_forecasts,
    market_forecasts,
  }: {
    plant_forecasts: StackForecasts;
    market_forecasts: MarketForecast[];
  } = $props();
</script>

<div class="flex flex-col overflow-y-auto">
  <div>
    <h2 class="text-xl">Clients et centrales</h2>
    {#each plant_forecasts as [plant_id, forecast] (plant_id)}
      <div>{forecast}</div>
    {/each}
  </div>
  <div>
    <h2 class="text-xl">Marché</h2>
    {#each market_forecasts as forecast}
      <div>{forecast.direction}: {forecast.volume}</div>
    {/each}
  </div>
</div>
