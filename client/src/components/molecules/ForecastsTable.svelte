<script lang="ts">
  import { PLANT_ICONS, PLANT_NAMES } from "$lib/label";
  import { type Forecasts } from "../organisms/Forecasts.svelte";

  let {
    total_forecasts,
    consumers_forecasts,
    renewable_forecasts,
  }: {
    total_forecasts: Forecasts;
    consumers_forecasts: Forecasts;
    renewable_forecasts: Forecasts;
  } = $props();
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
