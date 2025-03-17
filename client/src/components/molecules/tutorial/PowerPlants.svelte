<script lang="ts">
  import GenericPlant from "../GenericPlant.svelte";
  import Score from "../CurrentScore.svelte";
  import { none } from "$lib/Options";

  let base_position = 300;
  let base_pnl = 300 * 65;

  let position = $derived.by(() => {
    return setpoint - base_position;
  });
  let energy_cost = 60;
  let pnl = $derived.by(() => {
    return base_pnl - setpoint * energy_cost;
  });

  let setpoint = $state(0);
  let cost = $derived(setpoint * 60);
  const updateSetpoint = (new_setpoint: number) => {
    setpoint = Math.max(0, Math.min(new_setpoint, 700));
  };
</script>

<div class="px-4 sm:px-10 py-4 text-success-content bg-success rounded-md m-2">
  <Score {position} {pnl} periods={none()} />
</div>

<h2 class="px-4 pt-2 font-semibold">Pilotage des centrales 🔌</h2>
<p class="px-4 pt-2">
  Pour vous équilibrer vous disposez de centrales que vous pouvez piloter.
  Essayez de réduire votre déficit avec cette centrale !
</p>

<div class="p-5">
  <GenericPlant
    {cost}
    {setpoint}
    {updateSetpoint}
    dispatchable={true}
    max_setpoint={700}
    type={"gaz"}
    {energy_cost}
  />
</div>

<p class="px-4">Il existe plusieurs types de centrales :</p>
<ul class="px-4">
  <li>
    🔥 <i>centrale à gaz</i> : entièrement pilotable mais coûteuse à exploiter,
  </li>
  <li>
    ☢️ <i>centrale nucléaire</i> : peu chère, mais changer sa production la bloquera
    pour la période suivante,
  </li>
  <li>
    ☀️ <i>centrale solaire</i> : ne coûte rien mais a une production variable que
    vous ne pouvez contrôler,
  </li>
  <li>
    🔋 <i>batterie</i> : vous pouvez choisir de la charger ou décharger pour stocker
    de l'énergie d'une période à l'autre.
  </li>
</ul>
