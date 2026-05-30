<script lang="ts">
  import { none, some, type Option } from "$lib/Options";
  import PlantConfig from "../molecules/PlantConfig.svelte";

  let { payload = $bindable() }: { payload: Option<any> } = $props();

  let renewable_capacity = $state(300);
  let gas_capacity = $state(500);
  let gas_cost = $state(80);
  let nuclear_capacity = $state(1000);
  let nuclear_cost = $state(35);
  let battery_charge = $state(300);
  let number_of_periods = $state("6");

  let consumers_revenues = $state(56);
  let consumers_capacity = $state(1800);

  let areGasPlantOptionsValid = $derived.by(() => {
    const max_power_valid =
      !isNaN(Number(gas_capacity)) && Number(gas_capacity) > 0;

    const cost_valid = !isNaN(Number(gas_cost)) && Number(gas_cost) > 0;

    return max_power_valid && cost_valid;
  });

  let areNuclearPlantOptionsValid = $derived.by(() => {
    const max_power_valid =
      !isNaN(Number(nuclear_capacity)) && Number(nuclear_capacity) > 0;

    const cost_valid = !isNaN(Number(nuclear_cost)) && Number(nuclear_cost) > 0;

    return max_power_valid && cost_valid;
  });

  let areBatteryOptionsValid = $derived.by(() => {
    const max_charge_valid =
      !isNaN(Number(battery_charge)) && Number(battery_charge) > 0;

    return max_charge_valid;
  });

  let isNumberOfPeriodsValid = $derived(
    number_of_periods !== "" &&
      !isNaN(Number(number_of_periods)) &&
      Number(number_of_periods) > 0,
  );

  let areConsumersOptionsValid = $derived.by(() => {
    const capacity_valid =
      !isNaN(Number(consumers_capacity)) && Number(consumers_capacity) > 0;

    const revenues_valid =
      !isNaN(Number(consumers_revenues)) && Number(consumers_revenues) > 0;

    return capacity_valid && revenues_valid;
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
        Fixed: {
          gas_cost: Number(gas_cost),
          gas_capacity: Number(gas_capacity),
          nuclear_cost: Number(nuclear_cost),
          nuclear_capacity: Number(nuclear_capacity),
          battery_capacity: Number(battery_charge),
          consumers_capacity: -Number(consumers_capacity),
          consumers_revenues: Number(consumers_revenues),
          consumers_forecasts_range: Number(number_of_periods),
          renewable_capacity: Number(renewable_capacity),
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
      bind:capacity={renewable_capacity}
      plant="RenewablePlant"
      fixedPrice={true}
    />
    <PlantConfig
      bind:price={nuclear_cost}
      bind:capacity={nuclear_capacity}
      plant="Nuclear"
    />
    <PlantConfig
      bind:price={gas_cost}
      bind:capacity={gas_capacity}
      plant="GasPlant"
    />
    <PlantConfig
      price={0}
      bind:capacity={battery_charge}
      plant="Battery"
      fixedPrice={true}
    />
  </div>
</fieldset>
