<script lang="ts">
  const icons = {
    gaz: "🔥",
    renewable: "☀️",
    consumers: "🏙️",
  };
  const names = {
    gaz: "Centrale gaz",
    renewable: "Solaire",
    consumers: "Clients",
  };

  let {
    cost,
    max_setpoint,
    setpoint,
    updateSetpoint,
    energy_cost,
    dispatchable,
    type,
  }: {
    setpoint: number;
    max_setpoint: number;
    cost: number;
    energy_cost: number;
    updateSetpoint: (setpoint: number) => void;
    type: "gaz" | "renewable" | "consumers";
    dispatchable: boolean;
  } = $props();
  let current_setpoint = $state(0);
  $effect(() => {
    current_setpoint = type === "consumers" ? -setpoint : setpoint;
  });

  let debounceTimer: ReturnType<typeof setTimeout>;
  const debouncedUpdateSetpoint = () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      updateSetpoint(current_setpoint);
    }, 500);
  };

  let sliderClass = $derived.by(() => {
    let classNames =
      "range block my-auto w-full rounded-lg appearance-none cursor-pointer";
    classNames += dispatchable
      ? ""
      : " [--range-thumb:transparent] opacity-100!";
    return classNames;
  });
</script>

<div class="flex flex-row gap-1 w-full justify-stretch">
  <div class="self-center text-2xl">{icons[type]}</div>
  <div class="flex flex-col grow">
    <div class="flex flex-row justify-between">
      <div class="italic">{names[type]}</div>
      <div>
        {(-cost).toLocaleString("fr-FR", { signDisplay: "exceptZero" })} €
        {#if energy_cost !== 0}
          <span class="font-light italic">
            ({energy_cost.toLocaleString("fr-FR")} €/MWh)
          </span>
        {/if}
      </div>
    </div>

    <div class="grid grid-cols-[1fr_135px] p-1.5">
      <input
        class={sliderClass}
        type="range"
        disabled={!dispatchable}
        bind:value={current_setpoint}
        max={max_setpoint}
        step="25"
        oninput={debouncedUpdateSetpoint}
      />

      <div class="pl-2 justify-self-end">
        {current_setpoint.toLocaleString("fr-FR")} / {max_setpoint.toLocaleString(
          "fr-FR",
        )} MW
      </div>
    </div>
  </div>
</div>
