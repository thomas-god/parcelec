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
    </tbody>
  </table>
</div>
