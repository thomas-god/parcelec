<script lang="ts">
  import { PLANT_ICONS, PLANT_NAMES, type PlantType } from "$lib/label";

  let {
    price = $bindable(),
    capacity = $bindable(),
    plant,
    fixedPrice = false,
    capacity_suffix = "",
  }: {
    price: number;
    capacity: number;
    plant: PlantType;
    fixedPrice?: boolean;
    capacity_suffix?: string;
  } = $props();
</script>

<div class="flex flex-col gap-2 flex-wrap">
  <div class="divider divider-start flex flex-row items-center max-w-125">
    <img
      src={PLANT_ICONS[plant]}
      alt={PLANT_NAMES[plant] + "icon"}
      class="inline w-6 h-6"
    />
    {PLANT_NAMES[plant]}
  </div>
  <div class="flex flex-row gap-2 justify-between">
    <label class="fieldset-label flex flex-row items-center">
      <input
        type="number"
        min={0}
        class="input input-sm validator text-base"
        bind:value={() => price.toString(), (v) => (price = Number(v))}
        required
        disabled={fixedPrice}
      />
      <div class="shrink-0">€/MWh</div>
    </label>

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
  </div>
</div>
