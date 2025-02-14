<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";
  let name = $state("");
  let error = $state(false);

  const registerPlayer = async () => {
    let rest = await fetch(`${PUBLIC_APP_URL}/game/join`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      mode: "cors",
      credentials: "include",
      body: JSON.stringify({ name }),
    });
    if (rest.status === 201) {
      goto("/game");
    } else {
      name = "";
      error = true;
    }
  };
</script>

<div class="flex flex-col gap-3 m-6 mt-12">
  <h2 class="text-center text-2xl">Rejoindre une partie ⚡</h2>
  <label class="mx-auto pt-3 text-base">
    Votre pseudo
    <input bind:value={name} type="text" class="border border-amber-200 pl-2" />
  </label>
  {#if error}
    <p class="text-red-400 text-center text-sm">Pseudo déjà pris</p>
  {/if}
  <button
    onclick={() => registerPlayer()}
    class=" text-white bg-green-700 active:bg-green-700 hover:bg-green-800 font-medium rounded-lg text-base px-5 py-2.5 mx-auto my-2"
    >Rejoindre</button
  >
</div>
