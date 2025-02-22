<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";
  import Scores from "../../components/molecules/Scores.svelte";

  let error = $state(false);
  const startTutorial = async () => {
    let response = await fetch(`${PUBLIC_APP_URL}/tutorial`, {
      method: "POST",
      mode: "cors",
      credentials: "include",
    });
    if (response.status === 201) {
      goto("/game");
    } else {
      error = true;
    }
  };
  let index = $state(0);
  let position_values = [-1200, 250, 0];
  let position = $derived(position_values.at(index)!);
  let pnl_values = [3000, -550, 1200];
  let pnl = $derived(pnl_values.at(index)!);
  setInterval(() => {
    index = (index + 1) % position_values.length;
  }, 1500);
</script>

<div class="flex flex-col max-w-[500px] mx-auto text-justify">
  <h1 class="text-center font-semibold text-xl my-3">
    Bienvenue dans Parcelec !
  </h1>
  <div class="p-4">
    L'objectif de Parcelec est d'atteindre l'équilibre énergétique pour produire
    autant que ce vos clients 🏙️ consomment. Mais attention il vous faudra
    trouver l'équilibre au meilleur cout !
  </div>
  <div
    class="px-4 sm:px-10 py-4 text-neutral-content bg-neutral rounded-md m-2"
  >
    <Scores {position} {pnl} />
  </div>
  <p class="p-4">
    Pour cela, vous disposez de plusieurs sources d'énergie : les centrales à
    gaz 🔥, entièrement pilotables mais coûteuses à exploiter ; les énergies
    renouvelables ☀️, gratuites mais dont la production varie selon les périodes
    ; et les batteries 🔋 qui permettent de stocker l'énergie entre deux
    périodes.
  </p>
  <p class="p-4">
    Et ce n'est pas tout : vous avez accès au marché de l'énergie pour acheter
    et vendre selon vos besoins. Restez à l'affût des bonnes opportunités !
  </p>
  <div class="self-center">
    <button onclick={startTutorial} class="text-lg mt-3"
      >➡️ Commencer une partie</button
    >
  </div>
</div>

{#if error}
  <p>Erreur lors de la création du tutoriel</p>
{/if}
