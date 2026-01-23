<script lang="ts">
  import type {
    StackForecasts,
    StackHistory,
    StackSnapshot,
  } from "$lib/message";
  import ForecastsChart from "../molecules/ForecastsChart.svelte";
  // import ForecastsTable from "../molecules/ForecastsTable.svelte";

  let {
    plant_forecasts,
    plant_snapshots,
    history,
  }: {
    plant_forecasts: StackForecasts;
    plant_snapshots: StackSnapshot;
    history: StackHistory;
  } = $props();
  let chartWidth: number = $state(300);

  export type Forecast = NonNullable<
    StackForecasts extends Map<any, infer I> ? I : never
  >;
  export type ForecastValue = (Forecast extends Array<infer I>
    ? I
    : never)["value"];
  type PlantType = (StackSnapshot extends Map<any, infer I>
    ? I
    : never)["type"];
  export type History = NonNullable<
    StackHistory extends Map<any, infer I> ? I : never
  >;
  export type PlantOutput = History extends Array<infer I> ? I : never;

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

  let history_by_plant = $derived.by(() => {
    let merged: [string, PlantType, PlantOutput[]][] = [];
    for (const [id, values] of history.entries()) {
      if (!!values) {
        const plant = plant_snapshots.get(id);
        if (!!plant) {
          merged.push([id, plant.type, values]);
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

  let renewables_forecasts = $derived.by(() => {
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

  let consumers_history = $derived.by(() => {
    const history: number[] = [];
    // Get history
    for (const [_, plant, values] of history_by_plant) {
      if (plant === "Consumers") {
        for (const [idx, value] of values.entries()) {
          const previous = history.at(idx) || 0;
          history[idx] = previous + value.setpoint;
        }
      }
    }

    // Get current setpoint
    let total = 0;
    for (const [id, plant] of plant_snapshots) {
      if (plant.type === "Consumers") {
        total += plant.output.setpoint;
      }
    }
    history.push(total);
    return history;
  });

  let renewables_history = $derived.by(() => {
    const history: number[] = [];
    // Get history
    for (const [_, plant, values] of history_by_plant) {
      if (plant === "RenewablePlant") {
        for (const [idx, value] of values.entries()) {
          const previous = history.at(idx) || 0;
          history[idx] = previous + value.setpoint;
        }
      }
    }

    // Get current setpoint
    let total = 0;
    for (const [id, plant] of plant_snapshots) {
      if (plant.type === "RenewablePlant") {
        total += plant.output.setpoint;
      }
    }
    history.push(total);
    return history;
  });
</script>

<div>
  <div class="w-full" bind:clientWidth={chartWidth}>
    <ForecastsChart
      {consumers_forecasts}
      {consumers_history}
      {renewables_forecasts}
      {renewables_history}
      width={chartWidth}
      height={300}
    />
  </div>

  <!-- <ForecastsTable
    {total_forecasts}
    {renewables_forecasts}
    {consumers_forecasts}
  /> -->
</div>
