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

<div class="card max-w-96 mx-auto mt-6 bg-base-100 shadow-sm">
  <div class="card-body">
    {#await loadGames()}
      <div class="mt-32 loading loading-ring loading-xl self-center"></div>
    {:then}
      {#if games.length > 0}
        <ul class="list bg-base-100">
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
        <div class="flex flex-col w-full gap-5">
          <div class="text-center text-lg">Pas de parties en cours 😞</div>
          <button class="btn btn-neutral btn-lg">
            <a href="/game/new" class="self-center">➕ Créer une partie</a>
          </button>
        </div>
      {/if}
    {/await}
  </div>
</div>
