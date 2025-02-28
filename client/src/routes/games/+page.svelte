<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";
  import { z } from "zod";

  const GamesSchema = z.object({
    games: z.array(
      z.object({
        id: z.string(),
        name: z.string(),
      }),
    ),
  });

  let games: z.infer<typeof GamesSchema>["games"] = $state([]);

  const loadGames = async () => {
    let rest = await fetch(`${PUBLIC_APP_URL}/games`, {
      method: "GET",
    });
    if (rest.status === 200) {
      games = GamesSchema.parse(await rest.json()).games;
    }
  };

  const gotoGame = (game_id: string) => {
    goto(`/game/${game_id}/join`);
  };
</script>

<div class="mt-8 mx-6 flex flex-col">
  {#await loadGames()}
    <div class="mt-32 loading loading-ring loading-xl self-center"></div>
  {:then}
    {#if games.length > 0}
      <ul class="list bg-base-100 rounded-box shadow-md">
        <li class="p-4 pb-2 text-sm font-semibold tracking-wide">
          Parties ouvertes
        </li>

        {#each games as game (game.id)}
          <li class="list-row items-center">
            <div class="list-col-grow">{game.name}</div>
            <button
              class="btn btn-square btn-ghost text-lg"
              onclick={() => gotoGame(game.id)}
            >
              ▶️
            </button>
          </li>
        {/each}
      </ul>
    {:else}
      <div class="flex flex-col w-full gap-3 mt-16">
        <div class="text-center text-lg">Pas de parties en cours 😞</div>
        <a href="/game/new" class="self-center text-lg">➕ Créer une partie</a>
      </div>
    {/if}
  {/await}
</div>
