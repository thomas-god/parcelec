<script lang="ts">
  import type { BatteryState } from "$lib/message";

  let {
    battery,
    updateSetpoint,
  }: { battery: BatteryState; updateSetpoint: (setpoint: number) => void } =
    $props();

  let current_setpoint = $state(0);
  $effect(() => {
    current_setpoint = battery.output.setpoint;
    // String(Math.abs(battery.output.setpoint));
  });
  let current_charge_state = $state(true);
  $effect(() => {
    current_charge_state = battery.output.setpoint > 0;
  });

  let debounceTimer: ReturnType<typeof setTimeout>;
  const debouncedUpdateSetpoint = () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      updateSetpoint(
        current_setpoint,
        // Math.abs(Number.parseInt(current_setpoint)) *
        //   (current_charge_state ? 1 : -1),
      );
    }, 400);
  };
</script>

<div class="flex flex-row gap-1 w-full justify-stretch">
  <div class="self-center text-2xl">🔋</div>
  <div class="flex flex-col grow">
    <div class="flex flex-row justify-between">
      <div class="italic">Batteries</div>
      <div>nan</div>
    </div>

    <div class="flex flex-row justify-between p-1.5">
      <div class="flex flex-col gap-2">
        <input
          class="range block my-auto bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700"
          type="range"
          bind:value={current_setpoint}
          max={battery.max_charge}
          min={-battery.max_charge}
          step="25"
          oninput={debouncedUpdateSetpoint}
        />
        <progress
          class="progress"
          value={battery.charge}
          max={battery.max_charge}
        ></progress>
      </div>
      <div class="shrink-0 pl-2">
        {current_setpoint} /{battery.max_charge.toLocaleString("fr-FR")} MW
      </div>
    </div>
  </div>
</div>
