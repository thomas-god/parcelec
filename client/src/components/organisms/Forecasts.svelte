<script lang="ts">
  import type {
    MarketForecast,
    StackForecasts,
    StackSnapshot,
  } from "$lib/message";
  const icons: Record<plantType, string> = {
    GasPlant: "🔥",
    RenewablePlant: "☀️",
    Consumers: "🏙️",
    Battery: "🔋",
    Nuclear: "↩️",
  };
  const names: Record<plantType, string> = {
    GasPlant: "Centrale gaz",
    RenewablePlant: "Solaire",
    Consumers: "Clients",
    Battery: "Batterie",
    Nuclear: "Centrale nucléaire",
  };
  const levelsNames: Record<NonNullable<forecastType>, string> = {
    High: "haute",
    Low: "basse",
    Medium: "moyenne",
  };
  const levelsNamesMasculin: Record<NonNullable<forecastType>, string> = {
    High: "haut",
    Low: "bas",
    Medium: "moyen",
  };
  const directionName: Record<MarketForecast["direction"], string> = {
    Buy: "acheteur",
    Sell: "vendeur",
  };
  let {
    plant_forecasts,
    plant_snapshots,
    market_forecasts,
  }: {
    plant_forecasts: StackForecasts;
    plant_snapshots: StackSnapshot;
    market_forecasts: MarketForecast[];
  } = $props();

  type plantType = (StackSnapshot extends Map<any, infer I>
    ? I
    : never)["type"];
  type forecastType = StackForecasts extends Map<any, infer I> ? I : never;

  let forecast_per_plant = $derived.by(() => {
    let mapped_forecasts: Map<string, [plantType, NonNullable<forecastType>]> =
      new Map();
    for (const [plant_id, forecast] of plant_forecasts) {
      if (!!forecast && plant_snapshots.has(plant_id)) {
        const plant = plant_snapshots.get(plant_id)!;
        mapped_forecasts.set(plant_id, [plant.type, forecast]);
      }
    }
    return mapped_forecasts;
  });
</script>

<div class="flex flex-col overflow-y-auto gap-3">
  <div>
    <h2 class="text-xl">Clients et centrales</h2>
    {#each forecast_per_plant as [plant_id, [plant_type, forecast]] (plant_id)}
      <div>
        <span class="text-2xl">
          {icons[plant_type]}
        </span>
        {names[plant_type]}: la {plant_type === "Consumers"
          ? "consommation"
          : "production"} sera
        <span class="italic underline">{levelsNames[forecast]}</span>
      </div>
    {:else}
      <div>Pas de prévisions</div>
    {/each}
  </div>
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
        <span class="italic">
          (prix inconnu)
          <!-- ({!!forecast.price
            ? `prix ${levelsNamesMasculin[forecast.price]}`
            : "prix inconnu"}) -->
        </span>
      </div>
    {:else}
      <div>Pas de prévisions</div>
    {/each}
  </div>
</div>
