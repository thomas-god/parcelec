<script lang="ts">
  import type { BatteryState } from "$lib/message";

  let { battery }: { battery: BatteryState } = $props();

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
    let style = `background-image: repeating-linear-gradient(
    -45deg,
    transparent 0 3px,
    black 3px 6px
  ); opacity: 75%;`;
    if (battery.current_setpoint > 0) {
      // Discharge
      style += "grid-column-start: 2;";
      style += "grid-column-end: 3;";
      style += "z-index: 60;";
    } else if (battery.current_setpoint < 0) {
      // Charge
      style += "grid-column-start: 1;";
      style += "grid-column-end: 3;";
      style += "z-index: 40;";
    } else {
      // No setpoint
      style += "display: none;";
    }
    return style;
  });
  // $inspect(style);
  $inspect(grid_template);
</script>

<div class="flex flex-col @container max-w-[400px]">
  <!-- <div>🔋</div> -->
  <div
    class="mb-5 h-6 rounded-full bg-gray-200 grow grid grid-rows-1"
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
  <div class="flex flex-col @sm:flex-row @sm:justify-between">
    <div class="justify-self-start">
      Charge: {battery.charge} / {battery.max_charge} MWh
    </div>
    <div>
      Consigne: {battery.current_setpoint} MW
    </div>
  </div>
</div>
