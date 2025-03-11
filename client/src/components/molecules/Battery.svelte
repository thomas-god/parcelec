<script lang="ts">
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

<div class="flex flex-row gap-1 w-full justify-stretch">
  <div class="self-center text-2xl">🔋</div>
  <div class="flex flex-col grow">
    <div class="flex flex-row justify-between">
      <div class="italic">Batteries</div>
      <div>0 €</div>
    </div>

    <div class="grid grid-cols-[1fr_135px] p-1.5">
      <div class="flex flex-col gap-1">
        <input
          class="range block my-auto w-full rounded-lg appearance-none cursor-pointer"
          type="range"
          disabled={false}
          bind:value={current_setpoint}
          min={0}
          max={max_charge}
          step="25"
          oninput={debouncedUpdateSetpoint}
          data-testid="battery-input"
        />
      </div>

      <div class="pl-2 justify-self-end">
        {signed_current_setpoint.toLocaleString("fr-FR")} MW
      </div>
    </div>
    <div class="flex flex-row justify-between">
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
