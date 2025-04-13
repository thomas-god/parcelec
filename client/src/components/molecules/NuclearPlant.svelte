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

  const BREAKPOINT = "450px";

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
  <div
    class={`
      w-full grid
      @min-[450px]:grid-cols-[auto_1fr_auto_135px]
      @max-[450px]:grid-cols-[auto_1fr_1fr]
    `}
  >
    <div class="col-start-1 row-start-1 row-span-2 self-center text-2xl">
      ☢️
    </div>
    <div
      class={`
        row-start-1 col-start-2
        @min-[450px]:col-span-1
        @max-[450px]:col-span-2
      `}
    >
      <span class="italic pl-1.5"> Centrale nucléaire </span>
      {#if !dispatchable}
        🔒
      {:else if dispatchable && current_setpoint !== previous_setpoint}
        <button onclick={resetSetpoint}> ↩️ </button>
      {/if}
    </div>
    <div
      class={`
        @min-[450px]:row-start-1 @min-[450px]:col-start-3 @min-[450px]:col-span-2 @min-[450px]:text-end
        @max-[450px]:row-start-3 @max-[450px]:col-start-2 @max-[450px]:text-start
      `}
    >
      {(-cost).toLocaleString("fr-FR", { signDisplay: "exceptZero" })} €
      <span class="font-light italic">
        ({energy_cost.toLocaleString("fr-FR")} €/MWh)
      </span>
    </div>
    <div class="p-1.5 row-start-2 col-start-2 col-span-2">
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
    <div
      class={`
        text-end
        @min-[450px]:row-start-2 @min-[450px]:col-start-4 @min-[450px]:p-1.5
        @max-[450px]:row-start-3 @max-[450px]:col-start-3
    `}
    >
      {current_setpoint.toLocaleString("fr-FR")} / {max_setpoint.toLocaleString(
        "fr-FR",
      )} MW
    </div>
  </div>
</div>
