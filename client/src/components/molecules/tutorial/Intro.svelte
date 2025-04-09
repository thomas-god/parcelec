<script lang="ts">
  import Score from "../CurrentScore.svelte";

  let position_index = $state(0);
  let position_values = [-1200, 250, 0];
  let position = $derived(position_values.at(position_index)!);
  let pnl_values = [3000, -550, 1200];
  let pnl = $derived(pnl_values.at(position_index)!);
  setInterval(() => {
    position_index = (position_index + 1) % position_values.length;
  }, 1000);
</script>

<h1 class="text-center px-4 font-semibold text-2xl">
  Bienvenue dans Parcelec !
</h1>
<div class="p-4">
  Votre objectif est d'atteindre l'équilibre énergétique en produisant autant
  que ce vos <i>clients</i> 🏙️ consomment. Mais attention il vous faudra trouver
  l'équilibre au meilleur coût !
</div>

<div class="mx-6 mb-4 p-4 text-success-content bg-success rounded-md">
  <Score {position} {pnl} />
</div>
