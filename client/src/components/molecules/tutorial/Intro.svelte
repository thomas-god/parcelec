<script lang="ts">
  import Score from "../CurrentScore.svelte";

  let position_index = $state(0);
  let position_values = [-1200, 250, 0];
  let position = $derived(position_values.at(position_index)!);
  let pnl_values = [3000, -550, 1200];
  let pnl = $derived(pnl_values.at(position_index)!);
  setInterval(() => {
    position_index = (position_index + 1) % position_values.length;
  }, 1500);
</script>

<div class="p-4">
  Votre objectif est d'atteindre l'équilibre énergétique en produisant autant
  que ce vos <i>clients</i> 🏙️ consomment. Mais attention il vous faudra trouver
  l'équilibre au meilleur coût !
</div>

<div class="px-4 sm:px-10 py-4 text-success-content bg-success rounded-md m-2">
  <Score {position} {pnl} />
</div>
