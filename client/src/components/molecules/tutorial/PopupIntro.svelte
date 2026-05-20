<script lang="ts">
  let { open = $bindable() }: { open: boolean } = $props();
</script>

<dialog class="modal z-50" {open}>
  <div class="modal-box">
    <form method="dialog">
      <button
        class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
        onclick={() => (open = false)}>✕</button
      >
    </form>
    <div class="flex flex-col gap-3">
      <h3 class="text-lg font-bold">Bienvenue dans Parcelec !</h3>
      <p>
        Votre objectif est d'atteindre l'équilibre énergétique en produisant
        autant que ce vos clients <img
          src="/icons/consumers.svg"
          alt="Consumers home icon"
          class="w-6 h-6 inline"
        />
        consomment tout en limitant vos coûts
        <img
          src="/icons/coin.svg"
          alt="Consumers home icon"
          class="w-6 h-6 inline"
        /> !
      </p>

      <p>
        Vous disposez de centrales <img
          src="/icons/slider.svg"
          alt="Slider icon"
          class="w-6 h-6 inline"
        />
        que vous pouver piloter, d'un marché
        <img
          src="/icons/exchange.svg"
          alt="Currency exchange icon"
          class="w-6 h-6 inline"
        />
        sur lequel acheter et vendre de l'energie, et de prévisions
        <img
          src="/icons/crystal.svg"
          alt="Crystal ball icon"
          class="w-6 h-6 inline"
        /> pour vous aider à prendre vos décisions.
      </p>

      <p>
        Pour chaque module vous cliquez sur l'icone <img
          src="/icons/question-mark.svg"
          alt="Stack of book icon"
          class="w-6 h-6 inline"
        /> pour en apprendre plus.
      </p>
    </div>
  </div>

  <form method="dialog" class="modal-backdrop">
    <button onclick={() => (open = false)}>close</button>
  </form>
</dialog>
