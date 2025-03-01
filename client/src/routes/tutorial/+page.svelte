<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";
  import { set } from "zod";
  import Score from "../../components/molecules/CurrentScore.svelte";
  import GenericPlant from "../../components/molecules/GenericPlant.svelte";

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

  let setpoint = $state(0);
  let cost = $derived(setpoint * 60);
  const updateSetpoint = (new_setpoint: number) => {
    setpoint = Math.max(0, Math.min(new_setpoint, 700));
  };
</script>

<div class="flex flex-col max-w-[500px] mx-auto text-justify">
  <h1 class="text-center font-semibold text-xl my-3">
    Bienvenue dans Parcelec !
  </h1>
  <div class="p-4">
    Votre objectif est d'atteindre l'équilibre énergétique en produisant autant
    que ce vos <i>clients</i> 🏙️ consomment. Mais attention il vous faudra trouver
    l'équilibre au meilleur coût !
  </div>
  <div
    class="px-4 sm:px-10 py-4 text-success-content bg-success rounded-md m-2"
  >
    <Score {position} {pnl} />
  </div>
  <h2 class="px-4 pt-2 font-semibold">Pilotage des centrales 🔌</h2>
  <p class="p-4">
    Vous pouvez piloter plusieurs sources d'énergie. La première et la plus
    simple est une <i>centrale à gaz</i> 🔥: entièrement pilotable mais coûteuse
    à exploiter. À l'inverse, <i>votre centrale nucléaire</i> ☢️ coute peu cher,
    mais si vous la pilotez, elle sera bloquée la periode suivante ! Plus
    simple, votre <i>centrale solaire</i> ☀️ a une production variable que vous
    ne pouvez contrôler. Pour faire face à cette variabilité pour disposez d'une
    <i>batterie</i> 🔋 que vous pouvez choisir de charger ou décharger.
  </p>
  <div class="p-5">
    <GenericPlant
      {cost}
      {setpoint}
      {updateSetpoint}
      dispatchable={true}
      max_setpoint={700}
      type={"gaz"}
      energy_cost={60}
    />
  </div>
  <h2 class="px-4 pt-2 font-semibold">Le marché 💱</h2>
  <p class="p-4">
    Vous n'êtes pas tout seul dans le monde de Parcelec : vous pouvez acheter et
    vendre de l'énergie aux autres acteurs et joueurs en déposant des offres via
    l'onglet <i>marché </i> 💱. Si deux offres d'achat et de vente ont le même prix,
    alors la transaction se fait.
  </p>
  <h2 class="px-4 pt-2 font-semibold">Les prévisions 🔮</h2>
  <p class="p-4">
    Pour vous aider dans vos décisions un onglet <i>prédictions</i> 🔮 vous donnera
    une idée de ce qu'il pourra se passer à la prochaine période.
  </p>
  <h2 class="px-4 pt-2 font-semibold">Periodes de jeu et score</h2>
  <p class="p-4">
    Vous pouvez piloter vos centrales et utiliser le marché autant de fois que
    vous le souhaitez. Quand vous êtes satisfait de votre équilibrage, vous
    pouvez terminer la phase de préparation pour voir votre score et passez à la
    période suivante. Les scores de chaque période s'additionnent, il faudra
    penser aux périodes suivantes lors de vos calculs !
  </p>
  <div class="self-center">
    <button onclick={startTutorial} class="text-lg mt-3 mb-5"
      >➡️ Commencer une partie</button
    >
  </div>
</div>

{#if error}
  <p>Erreur lors de la création du tutoriel</p>
{/if}
