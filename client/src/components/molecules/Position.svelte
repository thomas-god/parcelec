<script lang="ts">
  import type { StackSnapshot, Trade } from "$lib/message";
  import { match } from "ts-pattern";

  let { plants, trades }: { plants: StackSnapshot; trades: Trade[] } = $props();
  let plants_position = $derived.by(() => {
    /// Cannot use plants.entries().reduce(/.../) on WebKit...
    let total = 0;
    for (const [_, plant] of plants.entries()) {
      total += match(plant)
        .with({ type: "Battery" }, (batt) => batt.current_setpoint)
        .with({ type: "GasPlant" }, (plant) => plant.setpoint)
        .with({ type: "RenewablePlant" }, (plant) => plant.setpoint)
        .with({ type: "Consumers" }, (consumers) => consumers.setpoint)
        .exhaustive();
    }
    return total;
  });

  let trades_position = $derived(
    trades.reduce(
      (acc, trade) =>
        acc + (trade.direction === "Buy" ? trade.volume : -trade.volume),
      0,
    ),
  );

  let position = $derived(plants_position + trades_position);
</script>

<div class="text-2xl text-center">
  {#if position > 0}
    ⚠️ Surplus d'énergie: {Math.abs(position)} MW
  {:else if position < 0}
    ⚠️ Manque d'énergie: {Math.abs(position)} MW
  {:else}
    ✅ A l'équilibre
  {/if}
  <!-- Equilibre: {plants_position + trades_position} MW -->
</div>
