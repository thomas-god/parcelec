<script lang="ts">
  import { goto } from "$app/navigation";
  import type { GameState } from "$lib/message";
  import { isSome, type Option } from "$lib/Options";
  import type { Periods } from "$lib/types";
  import Countdown from "../atoms/Countdown.svelte";

  let {
    game_state,
    periods,
    delivery_period_end,
    player_is_ready,
    make_player_ready,
  }: {
    game_state: GameState;
    player_is_ready: Option<boolean>;
    make_player_ready: () => void;
    delivery_period_end: Option<Date>;
    periods: Periods;
  } = $props();

  let title = $derived.by(() => {
    if (game_state === "Open") {
      return "En attente d'autres joueurs";
    } else if (game_state === "Running") {
      return `Période ${periods.current} sur ${periods.last}`;
    } else if (game_state === "PostDelivery") {
      return `Période ${periods.current} terminée`;
    } else {
      return "Partie terminée";
    }
  });

  let ctaBtnName = $derived.by(() => {
    if (game_state === "Open") {
      return "Commencer";
    } else if (game_state === "Running") {
      return "Terminer";
    } else if (game_state === "PostDelivery") {
      if (periods.current === periods.last) {
        return "Résultats";
      }
      return `Période suivante`;
    } else {
      return "";
    }
  });

  let width = $state(0);
</script>

<div
  class={`
        w-full sticky top-0 z-30 @sm:h-18
        text-success-content bg-success rounded-b-none rounded-t-none
        @sm:p-4 p-2
        `}
  bind:clientWidth={width}
>
  <div
    class="max-w-300 w-full mx-auto h-full flex flex-row items-center justify-between @sm:text-base text-sm text-center align-middle"
  >
    <div class="flex flex-row items-center justify-center gap-1.5">
      <button onclick={() => goto("/")}
        ><img
          src="/icons/home.svg"
          alt="homepage icon"
          class="inline h-6 w-6"
        /></button
      >
      {title}
    </div>
    {#if isSome(delivery_period_end)}
      <div class="flex flex-row items-center gap-1">
        <img
          src="/icons/hourglass.svg"
          alt="Hourglass icon"
          class="@sm:w-6 @sm:h-6 w-5 h-5 inline"
        />
        <Countdown end_at={delivery_period_end.value} />
      </div>
    {/if}
    {#if game_state !== "Ended"}
      {#if isSome(player_is_ready)}
        {#if !player_is_ready.value}
          <button class="btn @sm:btn-md btn-md" onclick={make_player_ready}>
            {ctaBtnName}
            <img
              src="/icons/arrow-next.svg"
              alt="Next arrow icon"
              class="@sm:w-6 @sm:h-6 w-4 h-4 inline"
            />
          </button>
        {:else}
          <div class="italic opacity-80 flex flex-row items-center gap-1">
            <div>
              {width > 384 ? "En attente des autres joueurs" : "En attente"}
            </div>
            <div class="loading loading-ring @ms:loading-sm loading-xs"></div>
          </div>
        {/if}
      {/if}
    {/if}
  </div>
</div>
