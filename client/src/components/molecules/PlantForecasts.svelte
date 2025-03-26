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

  let merged_forecasts = $derived.by(() => {
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

  let sum_by_period = $derived.by(() => {
    const sums = new Map<number, number>();

    // Initialize sums for all periods
    periods.forEach((period) => sums.set(period, 0));

    // Sum up values for each period across all forecasts
    for (const [_, __, forecast] of merged_forecasts) {
      for (const point of forecast) {
        const currentSum = sums.get(point.period) || 0;
        sums.set(point.period, currentSum + point.value.value);
      }
    }

    return sums;
  });

  let periods = $derived.by(() => {
    if (merged_forecasts.length > 0) {
      const [_, __, forecast] = merged_forecasts[0];
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
        {#each periods as period (`${forecasts}-head-${period}`)}
          <th class="text-center">{period}</th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each merged_forecasts as [id, plant, forecast] (id)}
        <tr>
          <th>
            <span>
              {PLANT_ICONS[plant]}
            </span>
            {PLANT_NAMES[plant]}
          </th>
          {#each forecast as point (`${id}-row-${point.period}`)}
            <td class="text-center"
              >{point.value.value.toLocaleString("fr-FR", {
                signDisplay: "exceptZero",
              })} MW</td
            >
          {/each}
        </tr>
      {/each}
      <tr class="font-bold">
        <th class="text-end">Total</th>
        {#each periods as period (`${forecasts}-total-${period}`)}
          <td class="text-center">
            {(sum_by_period.get(period) || 0).toLocaleString("fr-FR", {
              signDisplay: "exceptZero",
            })} MW
          </td>
        {/each}
      </tr>
    </tbody>
  </table>
</div>
