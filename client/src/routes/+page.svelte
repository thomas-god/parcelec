<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";
  import { isSome, none, some, type Option } from "$lib/Options";
  import { z } from "zod";

  const ActiveGameSchema = z.object({
    name: z.string(),
    state: z.string(),
  });

  type ActiveGame = z.infer<typeof ActiveGameSchema>;

  let activeGame: Option<ActiveGame> = $state(none());

  const getActiveGame = async () => {
    let rest = await fetch(`${PUBLIC_APP_URL}/game`, {
      method: "GET",
      credentials: "include",
    });
    if (rest.status === 200) {
      activeGame = some(ActiveGameSchema.parse(await rest.json()));
    } else {
      activeGame = none();
    }
  };
</script>

<div class="navbar bg-base-200">
  <div class="navbar-start"></div>
  <div class="navbar-center"></div>
  <div class="navbar-end">
    <a
      class="btn btn-ghost text-xl"
      href="https://github.com/thomas-god/parcelec"
      aria-label="GitHub repository"
    >
      <svg
        aria-hidden="true"
        class="octicon octicon-mark-github"
        height="24"
        version="1.1"
        viewBox="0 0 16 16"
        width="24"
        ><path
          fill-rule="evenodd"
          d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0 0 16 8c0-4.42-3.58-8-8-8z"
        ></path>
      </svg></a
    >
  </div>
</div>
<div class="hero bg-base-200">
  <div class="hero-content text-center">
    <div class="max-w-md">
      <h1 class="text-5xl font-bold">Bienvenue dans Parcelec ⚡</h1>
      <p class="py-6">
        Parcelec vous place dans le rôle d'un producteur d'électricité. Votre
        but ? Satisfaire la consommation de vos clients en utilisant
        astucieusement vos centrales et les marchés de l'électricité.
      </p>
      {#await getActiveGame() then}
        {#if isSome(activeGame)}
          <div class="divider font-semibold">Partie en cours</div>
          <div class="flex flex-row justify-center items-center gap-2">
            <div>
              {activeGame.value.name.includes("tutorial")
                ? "tutorial"
                : activeGame.value.name}
              <span class="italic">
                ({activeGame.value.state})
              </span>
            </div>
            <div>
              <button class="btn btn-primary" onclick={() => goto("/game")}
                >Rejoindre</button
              >
            </div>
          </div>
        {/if}
      {/await}
      <div class="divider font-semibold">Nouvelle partie</div>
      <div class="flex flex-row gap-2 justify-center">
        <a href="/tutorial">
          <button class="btn btn-primary"> 📖 Tutoriel </button>
        </a>
        <a href="/game/new">
          <button class="btn btn-primary"> ➕ Créer </button>
        </a>
        <a href="/games">
          <button class="btn btn-primary"> ▶️ Rejoindre </button>
        </a>
      </div>
    </div>
  </div>
</div>
