<script lang="ts">
import Game from "./components/pages/Game.svelte";
type State = "Home" | "Game";

let state = $state<State>("Home");
</script>

{#if state === "Home"}
  <button onclick={() => (state = "Game")}>Join Game</button>
{/if}

{#if state === "Game"}
  <Game />
{/if}
