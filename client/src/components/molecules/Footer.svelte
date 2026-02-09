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
  class={`
  footer footer-center
  @max-[600px]:fixed @max-[600px]:bottom-8 @max-[600px]:w-full
  @min-[600px]:rounded-b-md @min-[600px]:mx-auto @min-[600px]:max-w-125
  `}
>
  <button onclick={startGame} class="btn btn-primary btn-lg">
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
