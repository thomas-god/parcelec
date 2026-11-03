<script lang="ts">
  import { PLANT_ICONS, PLANT_NAMES } from "#lib/label.ts";

  let {
    capacity = $bindable(),
    stock = $bindable(),
    capacity_suffix = "",
  }: {
    capacity: number;
    stock: number;
    capacity_suffix?: string;
  } = $props();
</script>

<div class="flex flex-col gap-2 flex-wrap">
  <div class="divider divider-start flex flex-row items-center max-w-125">
    <img
      src={PLANT_ICONS.Hydro}
      alt={PLANT_NAMES.Hydro + "icon"}
      class="inline w-6 h-6"
    />
    {PLANT_NAMES.Hydro}
  </div>
  <div class="flex flex-row gap-2 justify-between">
    <label class="fieldset-label flex flex-row items-center">
      <input
        type="number"
        min={0}
        class="input input-sm validator text-base"
        bind:value={() => capacity.toString(), (v) => (capacity = Number(v))}
        required
      />
      <div class="shrink-0">MW {capacity_suffix}</div>
    </label>

    <label class="fieldset-label flex flex-row items-center">
      <input
        type="number"
        min={0}
        class="input input-sm validator text-base"
        bind:value={() => stock.toString(), (v) => (stock = Number(v))}
        required
      />
      <div class="shrink-0">MWh {capacity_suffix}</div>
    </label>
  </div>
</div>
