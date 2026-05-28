<script lang="ts">
  import { PLANT_ICONS, PLANT_NAMES } from "$lib/label";

  let {
    max_setpoint,
    setpoint,
    updateSetpoint,
    plant_type,
  }: {
    setpoint: number;
    max_setpoint: number;
    updateSetpoint: (setpoint: number) => void;
    plant_type: "Battery" | "GasPlant" | "RenewablePlant" | "Nuclear";
  } = $props();
  let current_setpoint = $state(0);
  $effect(() => {
    current_setpoint = setpoint;
  });

  let debounceTimer: ReturnType<typeof setTimeout>;
  const debouncedUpdateSetpoint = () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      updateSetpoint(current_setpoint);
    }, 500);
  };

  let sliderClass = $derived.by(() => {
    let classNames = `range block my-auto w-full rounded-lg appearance-none cursor-pointer [--range-bg:#e0d0b6] ${plant_type}`;
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
      <img src={PLANT_ICONS[plant_type]} alt="Gas plant icon" class="w-8 h-8" />
    </div>
    <div
      class={`
        row-start-1 col-start-2
        @min-[450px]:col-span-1
        @max-[450px]:col-span-2
      `}
    >
      <span class="italic pl-1.5"> {PLANT_NAMES[plant_type]}</span>
    </div>
    <div
      class={`
        @min-[450px]:row-start-1 @min-[450px]:col-start-3 @min-[450px]:col-span-2 @min-[450px]:text-end
        @max-[450px]:row-start-3 @max-[450px]:col-start-2 @max-[450px]:text-start
      `}
    ></div>
    <div class="p-1.5 row-start-2 col-start-2 col-span-2">
      <input
        class={sliderClass}
        type="range"
        bind:value={current_setpoint}
        max={max_setpoint}
        step="25"
        oninput={debouncedUpdateSetpoint}
        data-testid="gas-plant-input"
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
  </div>
</div>

<style>
  .GasPlant {
    color: var(--gas-background-color);
  }
  .Nuclear {
    color: var(--nuclear-background-color);
  }
  .Battery {
    color: var(--storage-background-color);
  }
  .RenewablePlant {
    color: var(--renewable-background-color);
  }
</style>
