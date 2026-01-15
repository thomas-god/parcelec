<script lang="ts">
  import type { StackForecasts, StackSnapshot } from "$lib/message";
  import PlantForecasts from "../PlantForecasts.svelte";

  let {
    forecasts,
    plants,
  }: { forecasts: StackForecasts; plants: StackSnapshot } = $props();
</script>

<h2 class="font-semibold text-lg mt-4 max-[500px]:pl-4">Prévisions 🔮</h2>

<p class="px-4 pt-2">
  Une partie se compose de plusieurs périodes pour lesquelles vous devrez, à
  chaque fois, équilibrer votre parc. Vous pouvez changer les consignes de vos
  centrales et utiliser le marhcé autant de fois que vous le souhaitez pendant
  une période.
</p>
<p class="px-4">
  Certaines de vos actions (comme choisir de charger votre batterie avec votre
  production excédentaire au lieu de la vendre) auront des répercussions sur les
  périodes suivantes.
</p>
<p class="px-4 pb-2">
  Pour vous aider dans vos décisions vous disposez d'un onglet <i>prévisions</i>
  🔮 qui vous donnera une idée des consommations et production pour les périodes suivantes.
  Mais attention ces prévisions peuvent se tromper !
</p>

<div class="my-1 mx-3 bg-base-100 rounded-lg">
  <PlantForecasts {forecasts} {plants} />
</div>
