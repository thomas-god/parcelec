<script lang="ts">
  import { PLANT_ICONS, PLANT_NAMES } from "$lib/label";
  import type { StackForecasts, StackSnapshot } from "$lib/message";

  let {
    forecasts,
    plants,
  }: { forecasts: StackForecasts; plants: StackSnapshot } = $props();

  export type Forecast = NonNullable<
    StackForecasts extends Map<any, infer I> ? I : never
  >;
  export type ForecastValue = (Forecast extends Array<infer I>
    ? I
    : never)["value"];
  type PlantType = (StackSnapshot extends Map<any, infer I>
    ? I
    : never)["type"];

  let forecasts_by_plant = $derived.by(() => {
    let merged: [string, PlantType, Forecast][] = [];
    for (const [id, forecast] of forecasts.entries()) {
      if (!!forecast) {
        const plant = plants.get(id);
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

<table class="table table-md max-w-96 mx-auto">
  <thead class="text-base-content">
    <tr>
      <th class="text-end px-0">Période</th>
      <th class="text-center"
        >{PLANT_ICONS["Consumers"]} {PLANT_NAMES["Consumers"]}</th
      >
      <th class="text-center px-3"
        >{PLANT_ICONS["RenewablePlant"]} {PLANT_NAMES["RenewablePlant"]}</th
      >
      <th class="text-center">Total</th>
    </tr>
  </thead>
  <tbody>
    {#each total_forecasts as [period, _]}
      <tr>
        <th class="text-end">
          {period}
        </th>
        <td class="text-center px-2">
          <div class="flex flex-col">
            <span class="font-semibold">
              {consumers_forecasts.get(period)!.value.toLocaleString("fr-FR", {
                signDisplay: "exceptZero",
              })} MW
            </span>
            <span class="italic">
              ± {consumers_forecasts
                .get(period)!
                .deviation.toLocaleString("fr-FR")} MW
            </span>
          </div>
        </td>
        <td class="text-center px-2">
          <div class="flex flex-col">
            <span class="font-semibold">
              {renewable_forecasts.get(period)!.value.toLocaleString("fr-FR", {
                signDisplay: "exceptZero",
              })} MW
            </span>
            <span class="italic">
              ± {renewable_forecasts
                .get(period)!
                .deviation.toLocaleString("fr-FR")} MW
            </span>
          </div>
        </td>
        <th class="text-center px-2">
          <div class="flex flex-col">
            <span class="font-bold">
              {total_forecasts.get(period)!.value.toLocaleString("fr-FR", {
                signDisplay: "exceptZero",
              })} MW
            </span>
            <span class="italic font-semibold">
              ± {total_forecasts.get(period)!.deviation.toLocaleString("fr-FR")}
              MW
            </span>
          </div>
        </th>
      </tr>
    {:else}
      <tr>
        <td colspan="4" class="text-center py-4">Pas de prévisions</td>
      </tr>
    {/each}
  </tbody>
</table>
