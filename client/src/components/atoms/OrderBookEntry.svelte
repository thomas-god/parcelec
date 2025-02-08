<script lang="ts">
  import { type OrderBookEntry } from "$lib/message";
  const {
    entry,
  }: {
    entry: OrderBookEntry;
  } = $props();

  const price_in_euros = $derived(entry.price / 100);
</script>

{#if entry.direction === "Buy"}
  <span
    class="flex justify-end p-2 shadow-md shadow-green-500 rounded animate-fade-in-scale {entry.owned
      ? 'font-bold'
      : 'font-normal'}"
  >
    {price_in_euros}€ ({entry.volume} MW)
  </span>
{:else}
  <span
    class="flex justify-between p-2 shadow-md shadow-red-500 rounded animate-fade-in-scale {entry.owned
      ? 'font-bold'
      : 'font-normal'}"
  >
    {price_in_euros}€ ({entry.volume} MW)
  </span>
{/if}
