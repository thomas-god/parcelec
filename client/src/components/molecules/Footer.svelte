<script lang="ts">
  import type { GameState } from "$lib/message";
  import type { Periods } from "$lib/types";

  let {
    game_state,
    startGame,
    player_is_ready,
    periods,
  }: {
    game_state: GameState;
    player_is_ready: boolean;
    startGame: () => void;
    periods: Periods;
  } = $props();
</script>

<footer
  class="footer fixed bottom-0 bg-success text-success-content rounded-t-md p-2 pb-4 w-screen max-w-[600px] flex flex-col items-center text-xl"
>
  <button onclick={startGame}>
    {#if game_state === "Running"}
      {#if player_is_ready}
        En attente des autres joueurs
      {:else}
        ➡️ Terminer la période ({periods.current}/{periods.last})
      {/if}
    {:else if game_state === "Open"}
      {#if player_is_ready}
        En attente des autres joueurs
      {:else}
        ➡️ Commencer la partie
      {/if}
    {:else if game_state === "Ended"}
      ➡️ Retour au menu
    {:else if player_is_ready}
      En attente des autres joueurs
    {:else if periods.current < periods.last}
      ➡️ Période suivante
    {:else}
      ➡️ Terminer la partie
    {/if}</button
  >
</footer>
