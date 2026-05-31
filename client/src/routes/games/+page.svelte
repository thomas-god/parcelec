<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";
  import { StackConfigSchema } from "$lib/message";
  import { z } from "zod";
  import Header from "../../components/molecules/Header.svelte";

  const GamesSchema = z.object({
    games: z.array(
      z.object({
        id: z.string(),
        name: z.string(),
        stack: StackConfigSchema,
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

<Header />

<div class="flex flex-col items-center mt-6 px-2">
  {#await loadGames()}
    <div class="mt-32 loading loading-ring loading-xl self-center"></div>
  {:then}
    {#if games.length > 0}
      <ul class="list w-full max-w-120 bg-base-100 shadow-sm rounded-lg">
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
              <img
                src="/icons/arrow-next.svg"
                alt="Arrow pointing to the right icon"
                class="w-6 h-6 inline"
              />
            </button>
          </li>
        {/each}
      </ul>
    {:else}
      <div class="flex flex-col w-full gap-5">
        <div class="text-center text-lg">Pas de parties en cours 😞</div>
        <a href="/game/new" class="self-center">
          <button class="btn btn-neutral btn-lg">
            <img
              src="/icons/plus.svg"
              alt="Plus sign icon"
              class="w-6 h-6 inline"
            /> Créer une partie
          </button>
        </a>
      </div>
    {/if}
  {/await}
</div>
