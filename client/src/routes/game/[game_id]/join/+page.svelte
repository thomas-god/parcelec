<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";
  import { page } from "$app/state";
  let player_name = $state("");
  let error = $state(false);

  const registerPlayer = async () => {
    let rest = await fetch(`${PUBLIC_APP_URL}/game/join`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      mode: "cors",
      credentials: "include",
      body: JSON.stringify({ game_id: page.params.game_id, player_name }),
    });
    if (rest.status === 201) {
      goto("/game");
    } else {
      // player_name = "";
      error = true;
    }
  };
</script>

<div class="flex flex-col w-full mt-6">
  <fieldset
    class="fieldset w-xs bg-base-200 border border-base-300 p-4 rounded-box self-center"
  >
    <legend class="fieldset-legend text-base">Rejoindre une partie</legend>

    <label class="fieldset-label flex-col"
      ><div class="self-start text-sm">Votre pseudo</div>
      <input
        type="text"
        class="input"
        bind:value={player_name}
        onkeypress={(key) => {
          if (key.code === "Enter") {
            registerPlayer();
          }
        }}
      />
    </label>
    {#if error}
      <p class="fieldset-label">Ce pseudo est déjà pris !</p>
    {/if}

    <button class="btn btn-neutral mt-4 text-base" onclick={registerPlayer}
      >Créer</button
    >
  </fieldset>
</div>
