<script lang="ts">
  import type { ReadinessStatus } from "$lib/message";

  let {
    player_name,
    readiness_status,
  }: { player_name: string; readiness_status: ReadinessStatus } = $props();

  let sorted_readines_status = $derived.by(() => {
    let sorted_status: { player: string; ready: boolean }[] = [];
    for (const [player, ready] of readiness_status) {
      sorted_status.push({ player, ready });
    }
    sorted_status.sort((a, b) => (a.player > b.player ? 1 : -1));
    return sorted_status;
  });
</script>

<ul class="list bg-base-100 rounded-box shadow-md mx-3">
  <li class="p-4 pb-2 text-xs opacity-60 tracking-wide">Joueurs</li>
  {#each sorted_readines_status as { player, ready } (player)}
    <li class="list-row">
      {#if player === player_name}
        <div class="list-col-grow font-semibold">
          {player}
        </div>
      {:else}
        <div class="list-col-grow">{player}</div>
      {/if}
      <div>
        {ready ? "✅" : "⌛"}
      </div>
    </li>
  {/each}
</ul>
