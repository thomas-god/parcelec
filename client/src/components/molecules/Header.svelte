<script lang="ts">
  import type { GameState } from "$lib/message";
  import type { Periods } from "$lib/types";
  import CurrentScore from "../../components/molecules/CurrentScore.svelte";

  let {
    game_state,
    pnl,
    position,
    periods,
  }: {
    game_state: GameState;
    position: number;
    pnl: number;
    periods: Periods;
  } = $props();
</script>

{#if game_state === "Running"}
  <CurrentScore {position} {pnl} />
{:else if game_state === "Open"}
  <div class="text-2xl text-center mx-auto">En attente d'autres joueurs</div>
{:else if game_state === "Ended"}
  <div class="text-2xl text-center mx-auto">Partie terminée !</div>
{:else}
  <div class="text-2xl text-center mx-auto">
    Période terminée ! ({periods.current}/{periods.last})
  </div>
{/if}
