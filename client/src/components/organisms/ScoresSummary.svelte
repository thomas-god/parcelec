<script lang="ts">
  import type { DeliveryPeriodDetailedScore } from "$lib/message";
  import type { SvelteMap } from "svelte/reactivity";

  let {
    current_period,
    detailed_scores,
  }: {
    current_period: number;
    detailed_scores: SvelteMap<number, DeliveryPeriodDetailedScore>;
  } = $props();

  let current_detailed_score = $derived(detailed_scores.get(current_period));

  const fmt = (s: number): string =>
    s.toLocaleString("fr-FR", {
      signDisplay: "exceptZero",
    });
</script>

<div class="overflow-x-auto self-center">
  {#if current_detailed_score !== undefined}
    {@const consumption_vol =
      current_detailed_score.consumers.volume +
      current_detailed_score.battery_charge.volume +
      current_detailed_score.market_sold.volume}
    {@const consumption_pnl =
      current_detailed_score.consumers.pnl +
      current_detailed_score.battery_charge.pnl +
      current_detailed_score.market_sold.pnl}
    {@const production_vol =
      current_detailed_score.renewables.volume +
      current_detailed_score.gas.volume +
      current_detailed_score.nuclear.volume +
      current_detailed_score.battery_discharge.volume +
      current_detailed_score.market_bought.volume}
    {@const production_pnl =
      current_detailed_score.renewables.pnl +
      current_detailed_score.gas.pnl +
      current_detailed_score.nuclear.pnl +
      current_detailed_score.battery_discharge.pnl +
      current_detailed_score.market_bought.pnl}
    <table class="table table-sm">
      <tbody>
        <!-- Consumption -->
        <tr>
          <th> Consommations </th>
        </tr>
        <tr>
          <td
            ><img
              src="/icons/consumers.svg"
              alt="house icon"
              class="icon"
            />Clients</td
          >
          <td class="value"
            >{fmt(current_detailed_score.consumers.volume)} MW</td
          >
          <td class="value">{fmt(current_detailed_score.consumers.pnl)} €</td>
        </tr>
        <tr>
          <td
            ><img
              src="/icons/storage.svg"
              alt="battery icon"
              class="icon"
            />Batteries - charge</td
          >
          <td class="value"
            >{fmt(current_detailed_score.battery_charge.volume)} MW</td
          >
          <td class="value"
            >{fmt(current_detailed_score.battery_charge.pnl)} €</td
          >
        </tr>
        <tr>
          <td
            ><img
              src="/icons/market.svg"
              alt="market exchange icon"
              class="icon"
            />Marché - ventes</td
          >
          <td class="value"
            >{fmt(current_detailed_score.market_sold.volume)} MW</td
          >
          <td class="value">{fmt(current_detailed_score.market_sold.pnl)} €</td>
        </tr>
        <tr>
          <th class="text-right"> Total </th>
          <th>{fmt(consumption_vol)} MW</th>
          <th>{fmt(consumption_pnl)} €</th>
        </tr>

        <!-- Production -->
        <tr>
          <th> Productions </th>
        </tr>
        <tr>
          <td
            ><img
              src="/icons/renewable.svg"
              alt="renewable wind turbine icon"
              class="icon"
            />Renouvelables</td
          >
          <td class="value"
            >{fmt(current_detailed_score.renewables.volume)} MW</td
          >
          <td class="value">{fmt(current_detailed_score.renewables.pnl)} €</td>
        </tr>
        <tr>
          <td
            ><img
              src="/icons/gas.svg"
              alt="flamme gas plant icon"
              class="icon"
            />Centrales gaz</td
          >
          <td class="value">{fmt(current_detailed_score.gas.volume)} MW</td>
          <td class="value">{fmt(current_detailed_score.gas.pnl)} €</td>
        </tr>
        <tr>
          <td
            ><img
              src="/icons/nuclear.svg"
              alt="nuclear power plant icon"
              class="icon"
            />Centrales nucléaires</td
          >
          <td class="value">{fmt(current_detailed_score.nuclear.volume)} MW</td>
          <td class="value">{fmt(current_detailed_score.nuclear.pnl)} €</td>
        </tr>
        <tr>
          <td
            ><img
              src="/icons/storage.svg"
              alt="battery icon"
              class="icon"
            />Batteries - décharge</td
          >
          <td class="value"
            >{fmt(current_detailed_score.battery_discharge.volume)} MW</td
          >
          <td class="value"
            >{fmt(current_detailed_score.battery_discharge.pnl)} €</td
          >
        </tr>
        <tr>
          <td
            ><img
              src="/icons/market.svg"
              alt="market exchange icon"
              class="icon"
            />Marché - achats</td
          >
          <td class="value"
            >{fmt(current_detailed_score.market_bought.volume)} MW</td
          >
          <td class="value"
            >{fmt(current_detailed_score.market_bought.pnl)} €</td
          >
        </tr>
        <tr>
          <th class="text-right"> Total </th>
          <th>{fmt(production_vol)} MW</th>
          <th>{fmt(production_pnl)} €</th>
        </tr>

        <!-- Imbalance -->
        <tr>
          <th class="text-right"
            ><img src="/icons/balance.svg" alt="balance icon" class="icon" /> Déséquilibre
          </th>
          <th>{fmt(current_detailed_score.imbalance.volume)} MW</th>
          <th>{fmt(current_detailed_score.imbalance.pnl)} €</th>
        </tr>

        <!-- Grand total -->
        <tr>
          <th> </th>
          <th> Total </th>
          <th
            >{fmt(
              current_detailed_score.imbalance.pnl +
                production_pnl +
                consumption_pnl,
            )} €</th
          >
        </tr>
      </tbody>
    </table>
  {/if}
</div>

<style>
  .icon {
    width: calc(var(--spacing) * 7);
    height: calc(var(--spacing) * 7);
    margin-right: calc(var(--spacing) * 2);
    display: inline;
  }

  .value {
    text-align: center;
  }
</style>
