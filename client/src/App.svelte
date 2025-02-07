<script lang="ts">
import Game from "./components/organisms/Game.svelte";
import JoinGame from "./components/organisms/JoinGame.svelte";
type State = "Home" | "Register" | "Game";

let state = $state<State>("Home");
</script>

{#if state === "Home"}
  <button onclick={() => (state = "Register")}>Join Game</button>
  <button onclick={() => (state = "Game")}>Game</button>
{/if}

{#if state === "Register"}
  <JoinGame />
{/if}

{#if state === "Game"}
  <Game />
{/if}
