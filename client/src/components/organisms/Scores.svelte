<script lang="ts">
  import type { SvelteMap } from "svelte/reactivity";

  let {
    scores,
    current_period,
  }: {
    current_period: number;
    scores: SvelteMap<
      number,
      {
        balance: number;
        pnl: number;
        imbalance_cost: number;
      }
    >;
  } = $props();

  let scores_descending = $derived(scores.entries().toArray().reverse());

  let total = $derived.by(() => {
    let res = 0;
    for (const [_, score] of scores.entries()) {
      res += score.pnl + score.imbalance_cost;
    }
    return res;
  });
</script>

<div class="overflow-x-auto self-center">
  <table class="table-xs min-[440px]:table">
    <thead>
      <tr class="text-xs min-[440px]:text-md">
        <th>Période</th>
        <th>Equilibre</th>
        <th>PnL</th>
        <th>Ecarts</th>
        <th>Net</th>
      </tr>
    </thead>
    <tbody>
      {#each scores_descending as [period, score] (period)}
        <tr class={period === current_period ? "bg-base-200" : ""}>
          <th>{period}</th>
          <td
            >{score.balance.toLocaleString("fr-FR", {
              signDisplay: "exceptZero",
            })} MW</td
          >
          <td
            >{score.pnl.toLocaleString("fr-FR", {
              signDisplay: "exceptZero",
            })} €</td
          >
          <td
            >{score.imbalance_cost.toLocaleString("fr-FR", {
              signDisplay: "exceptZero",
            })} €</td
          >
          <th
            >{(score.pnl + score.imbalance_cost).toLocaleString("fr-FR", {
              signDisplay: "exceptZero",
            })} €</th
          >
        </tr>
      {/each}
    </tbody>
    <tfoot>
      <tr>
        <td></td>
        <td></td>
        <td></td>
        <td>Total</td>
        <td class="font-bold"
          >{total.toLocaleString("fr-FR", {
            signDisplay: "exceptZero",
          })} €</td
        >
      </tr>
    </tfoot>
  </table>
</div>
