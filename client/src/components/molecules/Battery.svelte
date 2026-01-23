<script lang="ts">
  const BREAKPOINT = "450px";
  let {
    max_charge,
    charge,
    setpoint,
    updateSetpoint,
  }: {
    setpoint: number;
    charge: number;
    max_charge: number;
    updateSetpoint: (setpoint: number) => void;
  } = $props();
  let current_setpoint = $state(0);
  $effect(() => {
    current_setpoint = Math.abs(setpoint);
  });
  let current_charge_state = $state(false);
  $effect(() => {
    if (setpoint !== 0) {
      current_charge_state = setpoint < 0;
      return;
    }
    // Battery has no setpoint
    if (charge === 0) {
      // Battery is empty, set charge mode per default
      current_charge_state = true;
    } else if (charge === max_charge) {
      // Battery is full, set discharge mode per default
      current_charge_state = false;
    } else {
      // Set charge mode by default
      current_charge_state = true;
    }
  });
  let signed_current_setpoint = $derived(
    current_charge_state ? -current_setpoint : current_setpoint,
  );

  let debounceTimer: ReturnType<typeof setTimeout>;
  const debouncedUpdateSetpoint = () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      const new_setpoint = current_charge_state
        ? -current_setpoint
        : current_setpoint;
      if (new_setpoint === setpoint) {
        return;
      }
      updateSetpoint(
        current_charge_state ? -current_setpoint : current_setpoint,
      );
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
      <img src="/icons/storage.svg" alt="Battery icon" class="w-8 h-8" />
    </div>
    <div
      class={`
        row-start-1 col-start-2
        @min-[450px]:col-span-1
        @max-[450px]:col-span-2
      `}
    >
      <span class="italic pl-1.5">Batterie</span>
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
        max={max_charge}
        step="25"
        oninput={debouncedUpdateSetpoint}
        data-testid="battery-input"
      />
    </div>
    <div
      class={`
        text-end
        @min-[450px]:row-start-2 @min-[450px]:col-start-4 @min-[450px]:p-1.5
        @max-[450px]:row-start-3 @max-[450px]:col-start-3
    `}
    >
      {signed_current_setpoint.toLocaleString("fr-FR")} MW
    </div>
    <div
      class={`
        flex flex-row justify-between col-start-2 col-span-3 pl-1.5
        @min-[450px]:row-start-3
        @max-[450px]:row-start-4
    `}
    >
      <span>
        {charge} / {max_charge} MWh
        {#if setpoint !== 0}
          ({(-setpoint).toLocaleString("fr-FR", {
            signDisplay: "exceptZero",
          })} MWh)
        {/if}
      </span>
      <label class="swap text-right font-semibold">
        <input
          type="checkbox"
          bind:checked={current_charge_state}
          oninput={debouncedUpdateSetpoint}
        />
        <div class="swap-on">Charge 🔄</div>
        <div class="swap-off">Décharge 🔄</div>
      </label>
    </div>
  </div>
</div>

<style>
  .range {
    color: var(--storage-background-color);
  }
</style>
