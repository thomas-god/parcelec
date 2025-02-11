<script lang="ts">
  import type { BatteryState } from "$lib/message";

  let {
    battery,
    updateSetpoint,
  }: { battery: BatteryState; updateSetpoint: (setpoint: number) => void } =
    $props();

  let current_setpoint = $state(0);
  $effect(() => {
    current_setpoint = battery.current_setpoint;
  });

  let debounceTimer: ReturnType<typeof setTimeout>;
  const deboundedUpdateSetpoint = () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      updateSetpoint(current_setpoint);
    }, 500);
  };

  let current_charge_percent = $derived(
    Math.round((battery.charge / battery.max_charge) * 100),
  );
  let charge_variation_percent = $derived(
    Math.round((Math.abs(battery.current_setpoint) / battery.max_charge) * 100),
  );
  let grid_template = $derived.by(() => {
    if (battery.current_setpoint > 0) {
      // Discharge
      const col1 = current_charge_percent - charge_variation_percent;
      const col2 = charge_variation_percent;
      const col3 = 100 - current_charge_percent;
      return `grid-template-columns: ${col1}fr ${col2}fr ${col3}fr;`;
    } else if (battery.current_setpoint < 0) {
      // Charge
      const col1 = current_charge_percent;
      const col2 = charge_variation_percent;
      const col3 = 100 - charge_variation_percent - current_charge_percent;
      return `grid-template-columns: ${col1}fr ${col2}fr ${col3}fr;`;
    } else {
      // No setpoint

      const col1 = current_charge_percent;
      const col2 = 100 - current_charge_percent;
      return `grid-template-columns: ${col1}fr ${col2}fr;`;
    }
  });
  let current_charge_style = $derived.by(() => {
    if (battery.current_setpoint > 0) {
      // Discharge
      return `grid-column-start:1; grid-column-end:3; z-index: 50;`;
    } else if (battery.current_setpoint < 0) {
      // Charge
      return `grid-column-start:1; grid-column-end:2; z-index: 50;`;
    } else {
      // No setpoint
      return `grid-column-start:1; grid-column-end:2; z-index: 50;`;
    }
  });

  let delta_charge_style = $derived.by(() => {
    let style = "";
    if (battery.current_setpoint > 0) {
      // Discharge
      style += `background-image: repeating-linear-gradient(
        -45deg,
        transparent 0 3px,
        white 3px 6px);`;
      style += "opacity: 70%;";
      style += "grid-column-start: 2;";
      style += "grid-column-end: 3;";
      style += "z-index: 60;";
    } else if (battery.current_setpoint < 0) {
      // Charge
      style += `background-image: repeating-linear-gradient(
        -45deg,
        transparent 0 3px,
        oklch(0.705 0.213 47.604) 3px 6px);`;
      style += "opacity: 100%;";
      style += "grid-column-start: 1;";
      style += "grid-column-end: 3;";
      style += "z-index: 40;";
    } else {
      // No setpoint
      style += "display: none;";
    }
    return style;
  });
</script>

<div
  class="flex flex-col @container max-w-[400px] self-center"
  style="width: 100%;"
>
  <div class="flex flex-row">
    <div>🔋</div>
    <div
      class="mb-1 h-6 rounded-full bg-gray-200 grow grid grid-rows-1"
      style={grid_template}
    >
      <div
        class="h-6 rounded-full bg-orange-500 col-start-1 col-end-2"
        style={current_charge_style}
      ></div>
      <div
        class="h-6 rounded-full border-dotted"
        style={delta_charge_style}
      ></div>
    </div>
  </div>
  <div class="flex flex-col @xs:flex-row @xs:justify-between">
    <div>
      <label
        >Consigne
        <input
          type="number"
          bind:value={current_setpoint}
          oninput={deboundedUpdateSetpoint}
          class="max-w-[60px] text-center"
          step="10"
        />
        MW
      </label>
    </div>
    <div class="justify-self-start">
      <span class="hidden @sm:inline pr-1">Charge:</span>{battery.charge} / {battery.max_charge}
      MWh
    </div>
  </div>
</div>
