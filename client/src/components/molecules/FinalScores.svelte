<script lang="ts">
  import { goto } from "$app/navigation";
  import type { GameResults } from "$lib/message";

  let {
    final_scores,
    player_name,
  }: { player_name: string; final_scores: GameResults } = $props();
</script>

<div class="flex flex-col gap-5 items-center w-full">
  <ol class="list bg-base-100 rounded-box shadow-md w-full">
    {#each final_scores as score (score.player)}
      <li class="list-row items-center">
        <div class="text-4xl font-thin opacity-30 tabular-nums">
          {score.rank}
        </div>
        {#if score.player === player_name}
          <div class="font-semibold">
            {score.player}
          </div>
        {:else}
          <div>
            {score.player}
          </div>
        {/if}
        <div>
          {score.score.toLocaleString("fr-FR", {
            signDisplay: "exceptZero",
          })} €
          {#if score.tier === "Gold"}
            <!-- 🥇 -->
            ⭐⭐⭐
          {:else if score.tier === "Silver"}
            <!-- 🥈 -->
            ⭐⭐
          {:else if score.tier === "Bronze"}
            <!-- 🥉 -->
            ⭐
          {:else}
            👍
          {/if}
        </div>
      </li>
    {/each}
  </ol>

  <div>
    <button class="btn btn-primary" onclick={() => goto("/")}
      >Retourner à l'accueil
      <img
        src="/icons/arrow-next.svg"
        alt="arrow next icon"
        class="h-5 w-5 inline"
      />
    </button>
  </div>
</div>
