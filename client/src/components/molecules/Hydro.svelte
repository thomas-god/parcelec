<script lang="ts">
  let {
    max_stock,
    max_setpoint,
    stock,
    setpoint,
    updateSetpoint,
  }: {
    setpoint: number;
    stock: number;
    max_stock: number;
    max_setpoint: number;
    updateSetpoint: (setpoint: number) => void;
  } = $props();

  let current_setpoint = $state(0);
  $effect(() => {
    current_setpoint = setpoint;
  });

  // The reservoir can only be drained: the setpoint is bounded by both the turbine power and the
  // energy currently stored in the reservoir.
  let max_available = $derived(Math.min(max_setpoint, stock));

  let debounceTimer: ReturnType<typeof setTimeout>;
  const debouncedUpdateSetpoint = () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      if (current_setpoint === setpoint) {
        return;
      }
      updateSetpoint(current_setpoint);
    }, 500);
  };
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
      <img src="/icons/hydro.svg" alt="Hydro plant icon" class="w-8 h-8" />
    </div>
    <div
      class={`
        row-start-1 col-start-2
        @min-[450px]:col-span-1
        @max-[450px]:col-span-2
      `}
    >
      <span class="italic pl-1.5">Hydraulique</span>
    </div>
    <div
      class={`
        @min-[450px]:row-start-1 @min-[450px]:col-start-3 @min-[450px]:col-span-2 @min-[450px]:text-end
        @max-[450px]:row-start-3 @max-[450px]:col-start-2 @max-[450px]:text-start
        pl-1.5
      `}
    >
      0 €
    </div>
    <div class="p-1.5 row-start-2 col-start-2 col-span-2">
      <input
        class="range block my-auto w-full rounded-lg appearance-none cursor-pointer [--range-bg:#e0d0b6]"
        type="range"
        bind:value={current_setpoint}
        min={0}
        max={max_available}
        step="25"
        oninput={debouncedUpdateSetpoint}
        data-testid="hydro-input"
      />
    </div>
    <div
      class={`
        text-end
        @min-[450px]:row-start-2 @min-[450px]:col-start-4 @min-[450px]:p-1.5
        @max-[450px]:row-start-3 @max-[450px]:col-start-3
    `}
    >
      {current_setpoint.toLocaleString("fr-FR")} MW
    </div>
    <div
      class={`
        flex flex-row justify-between col-start-2 col-span-3 pl-1.5
        @min-[450px]:row-start-3
        @max-[450px]:row-start-4
    `}
    >
      <span>
        {stock} / {max_stock} MWh
        {#if setpoint !== 0}
          ({(-setpoint).toLocaleString("fr-FR", {
            signDisplay: "exceptZero",
          })} MWh)
        {/if}
      </span>
      <span class="text-right font-semibold">Débit</span>
    </div>
  </div>
</div>

<style>
  .range {
    color: var(--hydro-background-color, #61b2e4);
  }
</style>
