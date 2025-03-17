<script lang="ts">
  import type { MarketForecast } from "$lib/message";

  type forecastLevel = MarketForecast["volume"];

  let { market_forecasts }: { market_forecasts: MarketForecast[] } = $props();

  const levelsNamesMasculin: Record<forecastLevel, string> = {
    High: "haut",
    Low: "bas",
    Medium: "moyen",
  };
  const directionName: Record<MarketForecast["direction"], string> = {
    Buy: "acheteur",
    Sell: "vendeur",
  };
</script>

<div>
  <h2 class="text-xl">Marché</h2>
  {#each market_forecasts as forecast}
    <div>
      Il y aura un potentiel <span class="italic underline"
        >{directionName[forecast.direction]}
      </span>
      pour un volume
      <span class="italic underline">
        {levelsNamesMasculin[forecast.volume]}
      </span>
      <span class="italic"> (prix inconnu) </span>
    </div>
  {:else}
    <div>Pas de prévisions</div>
  {/each}
</div>
