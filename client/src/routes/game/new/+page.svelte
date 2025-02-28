<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";

  let game_name = $state("");
  const createGame = async () => {
    let rest = await fetch(`${PUBLIC_APP_URL}/game`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      mode: "cors",
      credentials: "include",
      body: JSON.stringify({ game_name }),
    });
    if (rest.status === 201) {
      let { game_id } = await rest.json();
      goto(`/game/${game_id}/join`);
    }
  };
</script>

<div class="flex flex-col w-full mt-6">
  <fieldset
    class="fieldset w-xs bg-base-200 border border-base-300 p-4 rounded-box self-center"
  >
    <legend class="fieldset-legend text-base">Créer une partie</legend>

    <label class="fieldset-label flex-col"
      ><div class="self-start text-sm">Nom de la partie</div>
      <input
        type="text"
        class="input"
        bind:value={game_name}
        onkeypress={(key) => {
          if (key.code === "Enter") {
            createGame();
          }
        }}
      />
    </label>

    <button class="btn btn-neutral mt-4 text-base" onclick={createGame}
      >Créer</button
    >
  </fieldset>
</div>
