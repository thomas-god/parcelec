<script lang="ts">
  import { PLANT_ICONS, PLANT_NAMES } from "$lib/label";
  import type { StackForecasts, StackSnapshot } from "$lib/message";

  let {
    forecasts,
    plants,
  }: { forecasts: StackForecasts; plants: StackSnapshot } = $props();

  type Forecast = NonNullable<
    StackForecasts extends Map<any, infer I> ? I : never
  >;
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
    const forecasts = new Map<number, number>();
    for (const [_, plant, forecast] of forecasts_by_plant) {
      if (plant === "Consumers") {
        for (const point of forecast) {
          const f = forecasts.get(point.period) || 0;
          forecasts.set(point.period, f + point.value.value);
        }
      }
    }
    return forecasts;
  });

  let renewable_forecasts = $derived.by(() => {
    const forecasts = new Map<number, number>();
    for (const [_, plant, forecast] of forecasts_by_plant) {
      if (plant === "RenewablePlant") {
        for (const point of forecast) {
          const f = forecasts.get(point.period) || 0;
          forecasts.set(point.period, f + point.value.value);
        }
      }
    }
    return forecasts;
  });

  let total_forecasts = $derived.by(() => {
    const sums = new Map<number, number>();

    // Initialize sums for all periods
    periods.forEach((period) => sums.set(period, 0));

    // Sum up values for each period across all forecasts
    for (const [_, __, forecast] of forecasts_by_plant) {
      for (const point of forecast) {
        const currentSum = sums.get(point.period) || 0;
        sums.set(point.period, currentSum + point.value.value);
      }
    }

    return sums;
  });

  let periods = $derived.by(() => {
    if (forecasts_by_plant.length > 0) {
      const [_, __, forecast] = forecasts_by_plant[0];
      return forecast.map((f) => f.period);
    }
    return [];
  });
</script>

<div class="overflow-x-auto">
  <table class="table table-sm">
    <thead>
      <tr>
        <th class="text-end">Période</th>
        <th class="text-center"
          >{PLANT_ICONS["Consumers"]} {PLANT_NAMES["Consumers"]}</th
        >
        <th class="text-center"
          >{PLANT_ICONS["RenewablePlant"]} {PLANT_NAMES["RenewablePlant"]}</th
        >
        <!-- <th class="text-center">Total</th> -->
      </tr>
    </thead>
    <tbody>
      {#each total_forecasts as [period, _]}
        <tr>
          <th class="text-end">
            {period}
          </th>
          <td class="text-center">
            {consumers_forecasts.get(period)!.toLocaleString("fr-FR", {
              signDisplay: "exceptZero",
            })} MW
          </td>
          <td class="text-center">
            {renewable_forecasts.get(period)!.toLocaleString("fr-FR", {
              signDisplay: "exceptZero",
            })} MW
          </td>
          <!-- <th class="text-center">
            {total_forecasts.get(period)!.toLocaleString("fr-FR", {
              signDisplay: "exceptZero",
            })} MW
          </th> -->
        </tr>
      {/each}
    </tbody>
  </table>
</div>
