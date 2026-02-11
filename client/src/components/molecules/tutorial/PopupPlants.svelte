<script lang="ts">
  let { open = $bindable() }: { open: boolean } = $props();
</script>

<dialog class="modal z-50" {open}>
  <div class="modal-box">
    <div class="flex flex-col gap-3">
      <h3 class="text-lg font-bold">
        Centrales et pilotage <img
          src="/icons/slider.svg"
          alt="Slider icon"
          class="w-6 h-6 inline"
        />
      </h3>
      <p>Il existe plusieurs types de centrales :</p>
      <ul class="list text-base gap-1.5">
        <li>
          <img
            src="/icons/gas.svg"
            alt="Gas flamme icon"
            class="h-6 w-6 inline"
          />
          <i>centrale à gaz</i> : entièrement pilotable mais coûteuse à exploiter,
        </li>
        <li>
          <img
            src="/icons/nuclear.svg"
            alt="Nuclear plant icon"
            class="h-6 w-6 inline"
          /> <i>centrale nucléaire</i> : peu chère, mais changer sa production la
          bloquera pour la période suivante,
        </li>
        <li>
          <img
            src="/icons/renewable.svg"
            alt="Wind turbine icon"
            class="h-6 w-6 inline"
          /> <i>centrale éolienne</i> : ne coûte rien mais a une production variable
          que vous ne pouvez contrôler,
        </li>
        <li>
          <img
            src="/icons/storage.svg"
            alt="Battery icon"
            class="h-6 w-6 inline"
          /> <i>batterie</i> : vous pouvez choisir de la charger ou décharger pour
          stocker de l'énergie d'une période à l'autre.
        </li>
      </ul>
    </div>
  </div>

  <form method="dialog" class="modal-backdrop">
    <button onclick={() => (open = false)}>close</button>
  </form>
</dialog>
