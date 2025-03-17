<script lang="ts">
  import type { GameState } from "$lib/message";
  import CurrentScore from "../../components/molecules/CurrentScore.svelte";
  let {
    game_state,
    pnl,
    position,
  }: {
    game_state: GameState;
    position: number;
    pnl: number;
  } = $props();
</script>

{#if game_state === "Running"}
  <CurrentScore {position} {pnl} />
{:else if game_state === "Open"}
  <div class="text-2xl text-center mx-auto">En attente d'autres joueurs</div>
{:else if game_state === "Ended"}
  <div class="text-2xl text-center mx-auto">Partie terminée !</div>
{:else}
  <div class="text-2xl text-center mx-auto">Période terminée !</div>
{/if}
