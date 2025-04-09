<script lang="ts">
  let {
    cost,
    max_setpoint,
    setpoint,
    previous_setpoint,
    updateSetpoint,
    energy_cost,
    dispatchable,
  }: {
    setpoint: number;
    previous_setpoint: number;
    max_setpoint: number;
    cost: number;
    energy_cost: number;
    updateSetpoint: (setpoint: number) => void;
    dispatchable: boolean;
  } = $props();
  let current_setpoint = $state(0);
  $effect(() => {
    current_setpoint = setpoint;
  });

  let debounceTimer: ReturnType<typeof setTimeout>;
  const debouncedUpdateSetpoint = () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      updateSetpoint(current_setpoint);
    }, 500);
  };

  const resetSetpoint = () => {
    updateSetpoint(previous_setpoint);
  };

  let sliderClass = $derived.by(() => {
    let classNames =
      "range block my-auto w-full rounded-lg appearance-none cursor-pointer";
    if (!dispatchable) {
      classNames += " [--range-thumb:transparent] opacity-100!";
      if (setpoint === 0) {
        classNames += " [--range-progress:transparent]";
      }
    }
    return classNames;
  });
</script>

<div class="flex flex-row gap-1 w-full justify-stretch @container">
  <div class="self-center text-2xl">☢️</div>

  <!-- Large screens: info are on the left of the slider -->
  <div class="flex flex-col grow @max-[450px]:hidden">
    <div class="flex flex-row justify-between">
      <div>
        <span class="italic"> Centrale nucléaire </span>
        {#if !dispatchable}
          🔒
        {:else if dispatchable && current_setpoint !== previous_setpoint}
          <button onclick={resetSetpoint}> ↩️ </button>
        {/if}
      </div>
      <div>
        {(-cost).toLocaleString("fr-FR", { signDisplay: "exceptZero" })} €
        <span class="font-light italic">
          ({energy_cost.toLocaleString("fr-FR")} €/MWh)
        </span>
      </div>
    </div>

    <div class="grid grid-cols-[1fr_135px] p-1.5">
      <input
        class={sliderClass}
        type="range"
        disabled={!dispatchable}
        bind:value={current_setpoint}
        max={max_setpoint}
        step="25"
        oninput={debouncedUpdateSetpoint}
        data-testid="nuclear-plant-input"
      />

      <div class="pl-2 justify-self-end">
        {current_setpoint.toLocaleString("fr-FR")} / {max_setpoint.toLocaleString(
          "fr-FR",
        )} MW
      </div>
    </div>
  </div>

  <!-- Small screens: info are below the slider -->
  <div class="flex flex-col grow @min-[450px]:hidden">
    <div>
      <span class="italic"> Centrale nucléaire </span>
      {#if !dispatchable}
        🔒
      {:else if dispatchable && current_setpoint !== previous_setpoint}
        <button onclick={resetSetpoint}> ↩️ </button>
      {/if}
    </div>
    <div class="p-1.5">
      <input
        class={sliderClass}
        type="range"
        disabled={!dispatchable}
        bind:value={current_setpoint}
        max={max_setpoint}
        step="25"
        oninput={debouncedUpdateSetpoint}
        data-testid="nuclear-plant-input"
      />
    </div>
    <div class="flex flex-row justify-between">
      <div>
        {(-cost).toLocaleString("fr-FR", { signDisplay: "exceptZero" })} €
        <span class="font-light italic">
          ({energy_cost.toLocaleString("fr-FR")} €/MWh)
        </span>
      </div>
      <div class="pl-2 justify-self-end">
        {current_setpoint.toLocaleString("fr-FR")} / {max_setpoint.toLocaleString(
          "fr-FR",
        )} MW
      </div>
    </div>
  </div>
</div>
