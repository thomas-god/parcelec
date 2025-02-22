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
    current_charge_state = setpoint < 0;
  });
  let signed_current_setpoint = $derived(
    current_charge_state ? -current_setpoint : current_setpoint,
  );
  $inspect(current_charge_state);

  let debounceTimer: ReturnType<typeof setTimeout>;
  const debouncedUpdateSetpoint = () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      console.log("charge state", current_charge_state);
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
          class="range block my-auto w-full bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700"
          type="range"
          disabled={false}
          bind:value={current_setpoint}
          min={0}
          max={max_charge}
          step="25"
          oninput={debouncedUpdateSetpoint}
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
      <label class="swap text-right">
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
