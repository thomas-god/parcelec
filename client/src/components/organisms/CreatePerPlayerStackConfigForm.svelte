<script lang="ts">
  import { none, some, type Option } from "$lib/Options";
  import PlantConfig from "../molecules/PlantConfig.svelte";

  let {
    payload = $bindable(),
    number_of_periods,
  }: { payload: Option<any>; number_of_periods: number } = $props();

  let gas_max_capacity = $state(500);
  let gas_cost = $state(80);
  let nuclear_max_capacity = $state(1000);
  let nuclear_cost = $state(35);
  let renewable_max_capacity = $state(1000);
  let consumers_capacity = $state(1000);
  let consumers_revenues = $state(56);
  let battery_max_charge = $state(300);

  let areGasPlantOptionsValid = $derived.by(() => {
    const max_power_valid =
      !isNaN(Number(gas_max_capacity)) && Number(gas_max_capacity) > 0;

    const cost_valid = !isNaN(Number(gas_cost)) && Number(gas_cost) > 0;

    return max_power_valid && cost_valid;
  });

  let areNuclearPlantOptionsValid = $derived.by(() => {
    const max_power_valid =
      !isNaN(Number(nuclear_max_capacity)) && Number(nuclear_max_capacity) > 0;

    const cost_valid = !isNaN(Number(nuclear_cost)) && Number(nuclear_cost) > 0;

    return max_power_valid && cost_valid;
  });

  let areBatteryOptionsValid = $derived.by(() => {
    const max_charge_valid =
      !isNaN(Number(battery_max_charge)) && Number(battery_max_charge) > 0;

    return max_charge_valid;
  });

  let isNumberOfPeriodsValid = $derived(
    !isNaN(Number(number_of_periods)) && Number(number_of_periods) > 0,
  );

  let areConsumersOptionsValid = $derived.by(() => {
    const max_capacity_valid =
      !isNaN(Number(consumers_capacity)) && Number(consumers_capacity) > 0;

    const revenues_valid =
      !isNaN(Number(consumers_revenues)) && Number(consumers_revenues) > 0;

    return max_capacity_valid && revenues_valid;
  });

  let isFormValid = $derived(
    areGasPlantOptionsValid &&
      areNuclearPlantOptionsValid &&
      areBatteryOptionsValid &&
      isNumberOfPeriodsValid &&
      areConsumersOptionsValid,
  );

  $effect(() => {
    if (isFormValid) {
      payload = some({
        PerPlayer: {
          gas_cost: Number(gas_cost),
          gas_max_capacity: Number(gas_max_capacity),
          nuclear_cost: Number(nuclear_cost),
          nuclear_max_capacity: Number(nuclear_max_capacity),
          battery_max_capacity: Number(battery_max_charge),
          consumers_revenues: Number(consumers_revenues),
          consumers_capacity: -Number(consumers_capacity),
          consumers_forecasts_range: Number(number_of_periods),
          renewable_max_capacity: Number(renewable_max_capacity),
          renewable_forecasts_range: Number(number_of_periods),
        },
      });
    } else {
      payload = none();
    }
  });
</script>

<fieldset class="fieldset bg-base-100">
  <div class="flex flex-col gap-2">
    <PlantConfig
      bind:price={consumers_revenues}
      bind:capacity={consumers_capacity}
      plant="Consumers"
    />
    <PlantConfig
      price={0}
      bind:capacity={renewable_max_capacity}
      plant="RenewablePlant"
      fixedPrice={true}
      capacity_suffix="max"
    />
    <PlantConfig
      bind:price={nuclear_cost}
      bind:capacity={nuclear_max_capacity}
      plant="Nuclear"
      capacity_suffix="max"
    />
    <PlantConfig
      bind:price={gas_cost}
      bind:capacity={gas_max_capacity}
      plant="GasPlant"
      capacity_suffix="max"
    />
    <PlantConfig
      price={0}
      bind:capacity={battery_max_charge}
      plant="Battery"
      fixedPrice={true}
      capacity_suffix="max"
    />
  </div>
</fieldset>
