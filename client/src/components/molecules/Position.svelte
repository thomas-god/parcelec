<script lang="ts">
  import type { StackSnapshot, Trade } from "$lib/message";
  import { match } from "ts-pattern";

  let { plants, trades }: { plants: StackSnapshot; trades: Trade[] } = $props();
  let plants_position = $derived(
    plants.entries().reduce(
      (acc, [_id, plant]) =>
        acc +
        match(plant)
          .with({ type: "Battery" }, (batt) => batt.current_setpoint)
          .with({ type: "GasPlant" }, (plant) => plant.setpoint)
          .exhaustive(),
      0,
    ),
  );

  let trades_position = $derived(
    trades.reduce(
      (acc, trade) =>
        acc + (trade.direction === "Buy" ? trade.volume : -trade.volume),
      0,
    ),
  );
</script>

<p>Position totale: {plants_position + trades_position} MW</p>
