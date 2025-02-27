<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";
  import Score from "../../components/molecules/CurrentScore.svelte";

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
  let position_index = $state(0);
  let position_values = [-1200, 250, 0];
  let position = $derived(position_values.at(position_index)!);
  let pnl_values = [3000, -550, 1200];
  let pnl = $derived(pnl_values.at(position_index)!);
  setInterval(() => {
    position_index = (position_index + 1) % position_values.length;
  }, 1500);
</script>

<div class="flex flex-col max-w-[500px] mx-auto text-justify">
  <h1 class="text-center font-semibold text-xl my-3">
    Bienvenue dans Parcelec !
  </h1>
  <div class="p-4">
    Votre objectif est d'atteindre l'équilibre énergétique en produisant autant
    que ce vos clients 🏙️ consomment. Mais attention il vous faudra trouver
    l'équilibre au meilleur coût !
  </div>
  <div
    class="px-4 sm:px-10 py-4 text-success-content bg-success rounded-md m-2"
  >
    <Score {position} {pnl} />
  </div>
  <h2 class="px-4 pt-2 font-semibold">Pilotage des centrales 🔌</h2>
  <p class="p-4">
    Vous disposez de plusieurs sources d'énergie que vous pouvez piloter. La
    première et la plus simple est une centrale à gaz 🔥 qui est entièrement
    pilotables mais coûteuse à exploiter. À l'inverse, votre centrale solaire ☀️
    ne coûte rien mais sa production est variable. Pour faire face à cette
    variabilité pour disposez d'une batterie 🔋 que vous pouvez charger ou
    décharger.
  </p>
  <h2 class="px-4 pt-2 font-semibold">Le marché 💱</h2>
  <p class="p-4">
    Vous n'êtes pas tout seul dans le monde de Parcelec, puisque vous pouvez
    acheter et vendre de l'énergie aux autres acteurs et joueurs via le marché.
  </p>
  <h2 class="px-4 pt-2 font-semibold">Les prévisions 🔮</h2>
  <p class="p-4">
    Enfin, pour vous aider dans vos décisions un onglet prédictions vous donnera
    une idée de ce qu'il pourra se passer à la prochaine période.
  </p>
  <h2 class="px-4 pt-2 font-semibold">Phases de jeu et score</h2>
  <p class="p-4">
    Vous pouvez piloter vos centrales et utiliser le marché autant de fois que
    vous le souhaitez. Une fois que vous êtes satisfait de votre équilibrage,
    vous pouvez terminer la phase en cours pour voir votre score et passez à la
    phase suivante. Les scores de chaque phase s'additionnent, il faudra penser
    aux phases suivantes lors de vos calculs !
  </p>
  <div class="self-center">
    <button onclick={startTutorial} class="text-lg mt-3 mb-2"
      >➡️ Commencer une partie</button
    >
  </div>
</div>

{#if error}
  <p>Erreur lors de la création du tutoriel</p>
{/if}
