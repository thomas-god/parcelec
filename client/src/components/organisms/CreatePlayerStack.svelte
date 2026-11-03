<script lang="ts">
  import type { PerPlayerStackConfig, StackConfig } from "$lib/message";
  import Slider from "../molecules/Slider.svelte";

  let {
    config,
    sendMessage,
  }: {
    config: PerPlayerStackConfig;
    sendMessage: (payload: string) => void;
  } = $props();

  let gas_capacity = $state(0);
  let nuclear_capacity = $state(0);
  let renewable_capacity = $state(0);
  let battery_capacity = $state(0);
  let hydro_capacity = $state(0);
  let hydro_stock = $state(0);

  let total = $derived(
    gas_capacity +
      nuclear_capacity +
      renewable_capacity +
      battery_capacity +
      hydro_capacity,
  );
  let valid = $derived(total > 0);

  const createStack = () => {
    sendMessage(
      JSON.stringify({
        RegisterPlayerStackConfig: {
          gas_capacity,
          nuclear_capacity,
          battery_capacity,
          hydro_capacity,
          hydro_stock,
          renewable_capacity,
        },
      }),
    );
  };
</script>

<div class="card bg-base-100 shadow-sm mx-3">
  <div class="card-body px-2">
    <h2 class="text-xl font-bold text-center">Configurer vos centrales</h2>
    <Slider
      max_setpoint={config.gas_max_capacity}
      setpoint={gas_capacity}
      updateSetpoint={(v) => (gas_capacity = v)}
      plant_type={"GasPlant"}
    />
    <Slider
      max_setpoint={config.nuclear_max_capacity}
      setpoint={nuclear_capacity}
      updateSetpoint={(v) => (nuclear_capacity = v)}
      plant_type={"Nuclear"}
    />
    <Slider
      max_setpoint={config.battery_max_capacity}
      setpoint={battery_capacity}
      updateSetpoint={(v) => (battery_capacity = v)}
      plant_type={"Battery"}
    />
    <Slider
      max_setpoint={config.renewable_max_capacity}
      setpoint={renewable_capacity}
      updateSetpoint={(v) => (renewable_capacity = v)}
      plant_type={"RenewablePlant"}
    />
    <Slider
      max_setpoint={config.hydro_max_capacity}
      setpoint={hydro_capacity}
      updateSetpoint={(v) => (hydro_capacity = v)}
      plant_type={"Hydro"}
      label="Hydraulique - turbine"
    />
    <Slider
      max_setpoint={config.hydro_max_stock}
      setpoint={hydro_stock}
      updateSetpoint={(v) => (hydro_stock = v)}
      plant_type={"Hydro"}
      label="Hydraulique - stock"
      unit="MWh"
    />

    <div class="text-right italic">
      Total : {total.toLocaleString("fr-FR")} MW
    </div>
    <div class="text-right italic">
      Puissance typique de vos clients : {Math.abs(
        config.consumers_capacity,
      ).toLocaleString("fr-FR")} MW
    </div>

    <button class="btn btn-primary mx-2" onclick={createStack} disabled={!valid}
      >Créer
    </button>
  </div>
</div>
