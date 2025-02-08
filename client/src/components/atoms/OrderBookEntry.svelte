<script lang="ts">
  const {
    price,
    volume,
  }: {
    price: number;
    volume: number;
  } = $props();

  const price_in_euros = $derived(price / 100);
</script>

<span>
  {price_in_euros}€ ({volume} MW)
</span>
