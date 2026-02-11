<script lang="ts">
  let { open = $bindable() }: { open: boolean } = $props();
</script>

<dialog class="modal z-50" {open}>
  <div class="modal-box">
    <div class="flex flex-col gap-3">
      <h3 class="text-lg font-bold">
        Prévisions <img
          src="/icons/crystal.svg"
          alt="Crystal ball icon"
          class="w-6 h-6 inline"
        />
      </h3>
      <p class="pt-2">
        Une partie se compose de plusieurs périodes pour lesquelles vous devrez,
        à chaque fois, équilibrer votre parc. Vous pouvez changer les consignes
        de vos centrales et utiliser le marhcé autant de fois que vous le
        souhaitez pendant une période.
      </p>
      <p class="">
        Certaines de vos actions (comme choisir de charger votre batterie avec
        votre production excédentaire au lieu de la vendre) auront des
        répercussions sur les périodes suivantes.
      </p>
      <p class="pb-2">
        Pour vous aider dans vos décisions vous disposez d'un onglet <i
          >prévisions</i
        >
        qui vous donnera une idée des consommations et production pour les périodes
        suivantes. Mais attention ces prévisions peuvent se tromper !
      </p>
    </div>
  </div>

  <form method="dialog" class="modal-backdrop">
    <button onclick={() => (open = false)}>close</button>
  </form>
</dialog>
