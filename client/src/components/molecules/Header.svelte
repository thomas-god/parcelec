<script lang="ts">
  import type { GameState } from "$lib/message";
  import type { Periods } from "$lib/types";

  let {
    game_state,
    periods,
  }: {
    game_state: GameState;
    periods: Periods;
  } = $props();
</script>

{#if game_state !== "Running"}
  <div
    class={`
        max-w-150 w-full mx-auto
        sticky top-0 px-2 pt-5 pb-5 z-30
        text-success-content bg-success rounded-b-md
        @sm:p-6
        @min-[600px]:mt-8 @min-[600px]:rounded-t-md
        `}
  >
    {#if game_state === "Open"}
      <div class="text-2xl text-center mx-auto">
        En attente d'autres joueurs
      </div>
    {:else if game_state === "Ended"}
      <div class="text-2xl text-center mx-auto">Partie terminée !</div>
    {:else}
      <div class="text-2xl text-center mx-auto">
        Période terminée ! ({periods.current}/{periods.last})
      </div>
    {/if}
  </div>
{/if}
