<script lang="ts">
  let { position, pnl }: { position: number; pnl: number } = $props();
</script>

<div class="flex flex-row justify-around text-xl min-[430px]:text-2xl">
  <div class="text-left grow">
    {#if position > 0}
      ⚠️ Surplus : {Math.abs(position)} MW
    {:else if position < 0}
      ⚠️ Déficit : {Math.abs(position)} MW
    {:else}
      ✅ A l'équilibre
    {/if}
  </div>
  <div class="@container-normal grow text-right">
    <span class="hidden @3xs:inline"> Score : </span>
    <span class="inline @3xs:hidden"> 💰</span>
    {pnl.toLocaleString("fr-FR")} €
  </div>
</div>
