<script lang="ts">
  import { isSome, type Option } from "$lib/Options";

  export interface Periods {
    current: number;
    last: number;
  }
  let {
    position,
    pnl,
    periods,
  }: { position: number; pnl: number; periods: Option<Periods> } = $props();
</script>

<div class="flex flex-col">
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
      {pnl.toLocaleString("fr-FR")} €
      <span class="inline @3xs:hidden"> 💰</span>
    </div>
  </div>
  {#if isSome(periods)}
    <div class="italic text-center">
      Période {periods.value.current}/{periods.value.last}
    </div>
  {/if}
</div>
