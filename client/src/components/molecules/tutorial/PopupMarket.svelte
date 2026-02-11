<script lang="ts">
  let { open = $bindable() }: { open: boolean } = $props();
</script>

<dialog class="modal z-50" {open}>
  <div class="modal-box">
    <div class="flex flex-col gap-3">
      <h3 class="text-lg font-bold">
        Marché <img
          src="/icons/exchange.svg"
          alt="Currency exchange icon"
          class="w-6 h-6 inline"
        />
      </h3>
      <p class="pt-2">
        En plus de vos centrales, le <i>marché </i> vous permet d'acheter ou de vendre
        de l'énergie avec d'autres joueurs.
      </p>

      <p>
        Si deux offres d'achat et de vente ont le même prix, alors la
        transaction se fait. Sinon, les 2 offres restent sur le marché jusqu'à
        ce que quelqu'un dépose une nouvelle offre ou que la période se termine.
      </p>
    </div>
  </div>

  <form method="dialog" class="modal-backdrop">
    <button onclick={() => (open = false)}>close</button>
  </form>
</dialog>
