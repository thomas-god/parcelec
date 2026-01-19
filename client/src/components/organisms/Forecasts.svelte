<script lang="ts">
  import type { StackForecasts, StackSnapshot } from "$lib/message";
  import ForecastsTable from "../molecules/ForecastsTable.svelte";

  let {
    plant_forecasts,
    plant_snapshots,
  }: {
    plant_forecasts: StackForecasts;
    plant_snapshots: StackSnapshot;
  } = $props();

  export type Forecast = NonNullable<
    StackForecasts extends Map<any, infer I> ? I : never
  >;
  export type ForecastValue = (Forecast extends Array<infer I>
    ? I
    : never)["value"];
  type PlantType = (StackSnapshot extends Map<any, infer I>
    ? I
    : never)["type"];

  export type Forecasts = Map<
    number,
    {
      value: number;
      deviation: number;
    }
  >;

  let forecasts_by_plant = $derived.by(() => {
    let merged: [string, PlantType, Forecast][] = [];
    for (const [id, forecast] of plant_forecasts.entries()) {
      if (!!forecast) {
        const plant = plant_snapshots.get(id);
        if (!!plant) {
          merged.push([id, plant.type, forecast]);
        }
      }
    }
    return merged;
  });

  let consumers_forecasts = $derived.by(() => {
    const forecasts = new Map<number, ForecastValue>();
    for (const [_, plant, forecast] of forecasts_by_plant) {
      if (plant === "Consumers") {
        for (const point of forecast) {
          const f = forecasts.get(point.period) || { deviation: 0, value: 0 };
          forecasts.set(point.period, {
            value: f.value + point.value.value,
            deviation: f.deviation + point.value.deviation,
          });
        }
      }
    }
    return forecasts;
  });

  let renewable_forecasts = $derived.by(() => {
    const forecasts = new Map<number, ForecastValue>();
    for (const [_, plant, forecast] of forecasts_by_plant) {
      if (plant === "RenewablePlant") {
        for (const point of forecast) {
          const f = forecasts.get(point.period) || { deviation: 0, value: 0 };
          forecasts.set(point.period, {
            value: f.value + point.value.value,
            deviation: f.deviation + point.value.deviation,
          });
        }
      }
    }
    return forecasts;
  });

  let total_forecasts = $derived.by(() => {
    const forecasts = new Map<number, ForecastValue>();
    for (const [_, __, forecast] of forecasts_by_plant) {
      for (const point of forecast) {
        const f = forecasts.get(point.period) || { deviation: 0, value: 0 };
        forecasts.set(point.period, {
          value: f.value + point.value.value,
          deviation: f.deviation + point.value.deviation,
        });
      }
    }

    return forecasts;
  });
</script>

<div class="flex flex-col overflow-y-auto gap-3">
  <ForecastsTable
    {total_forecasts}
    {renewable_forecasts}
    {consumers_forecasts}
  />
</div>
