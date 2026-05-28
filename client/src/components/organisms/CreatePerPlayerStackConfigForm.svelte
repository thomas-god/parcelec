<script lang="ts">
  import { PLANT_ICONS, PLANT_NAMES } from "$lib/label";
  import { none, some, type Option } from "$lib/Options";
  import NumericInput from "../atoms/NumericInput.svelte";

  let { payload = $bindable() }: { payload: Option<any> } = $props();

  let gas_max_capacity = $state("500");
  let gas_cost = $state("80");
  let nuclear_max_capacity = $state("1000");
  let nuclear_cost = $state("35");
  let renewable_max_capacity = $state("1000");
  let consumers_capacity = $state("1000");
  let consumers_revenues = $state("56");
  let battery_max_charge = $state("300");
  let number_of_periods = $state("6");

  let areGasPlantOptionsValid = $derived.by(() => {
    const max_power_valid =
      gas_max_capacity !== "" &&
      !isNaN(Number(gas_max_capacity)) &&
      Number(gas_max_capacity) > 0;

    const cost_valid =
      gas_cost !== "" && !isNaN(Number(gas_cost)) && Number(gas_cost) > 0;

    return max_power_valid && cost_valid;
  });

  let areNuclearPlantOptionsValid = $derived.by(() => {
    const max_power_valid =
      nuclear_max_capacity !== "" &&
      !isNaN(Number(nuclear_max_capacity)) &&
      Number(nuclear_max_capacity) > 0;

    const cost_valid =
      gas_cost !== "" &&
      !isNaN(Number(nuclear_cost)) &&
      Number(nuclear_cost) > 0;

    return max_power_valid && cost_valid;
  });

  let areBatteryOptionsValid = $derived.by(() => {
    const max_charge_valid =
      battery_max_charge !== "" &&
      !isNaN(Number(battery_max_charge)) &&
      Number(battery_max_charge) > 0;

    return max_charge_valid;
  });

  let isNumberOfPeriodsValid = $derived(
    number_of_periods !== "" &&
      !isNaN(Number(number_of_periods)) &&
      Number(number_of_periods) > 0,
  );

  let areConsumersOptionsValid = $derived.by(() => {
    const max_capacity_valid =
      consumers_capacity !== "" &&
      !isNaN(Number(consumers_capacity)) &&
      Number(consumers_capacity) > 0;

    const revenues_valid =
      consumers_revenues !== "" &&
      !isNaN(Number(consumers_revenues)) &&
      Number(consumers_revenues) > 0;

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
          consumers_capacity: Number(consumers_capacity),
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
  <div class="join join-vertical bg-base-100">
    <!-- CONSUMERS OPTIONS -->
    <div class="collapse collapse-arrow join-item border-base-300 border">
      <input type="checkbox" />
      <div
        class="collapse-title font-semibold flex flex-row items-center gap-1"
      >
        <img
          src={PLANT_ICONS["Consumers"]}
          alt={PLANT_NAMES["Consumers"] + "icon"}
          class="inline w-6 h-6"
        />
        {PLANT_NAMES["Consumers"]}
      </div>
      <div class="collapse-content text-sm">
        <NumericInput
          title="Puissance maximale consommée par les clients (en MW)"
          error_message="La puissance doit être > 0"
          min_value="1"
          bind:value={consumers_capacity}
        />

        <NumericInput
          title="Revenus (en €/MWh)"
          error_message="Le revenue doit être >= 0"
          min_value="0"
          bind:value={consumers_revenues}
        />
      </div>
    </div>

    <!-- RENEWABLE PLANT OPTIONS -->
    <div class="collapse collapse-arrow join-item border-base-300 border">
      <input type="checkbox" />
      <div
        class="collapse-title font-semibold flex flex-row items-center gap-1"
      >
        <img
          src={PLANT_ICONS["RenewablePlant"]}
          alt={PLANT_NAMES["RenewablePlant"] + "icon"}
          class="inline w-6 h-6"
        />
        {PLANT_NAMES["RenewablePlant"]}
      </div>
      <div class="collapse-content text-sm">
        <NumericInput
          title="Puissance maximale possible (en MW)"
          error_message="La puissance doit être > 0"
          min_value="1"
          bind:value={renewable_max_capacity}
        />
      </div>
    </div>

    <!-- GAS PLANT OPTIONS -->
    <div class="collapse collapse-arrow join-item border-base-300 border">
      <input type="checkbox" />
      <div
        class="collapse-title font-semibold flex flex-row items-center gap-1"
      >
        <img
          src={PLANT_ICONS["GasPlant"]}
          alt={PLANT_NAMES["GasPlant"] + "icon"}
          class="inline w-6 h-6"
        />
        {PLANT_NAMES["GasPlant"]}
      </div>
      <div class="collapse-content text-sm">
        <NumericInput
          title="Puissance maximale possible (en MW)"
          error_message="La puissance doit être > 0"
          min_value="1"
          bind:value={gas_max_capacity}
        />
        <NumericInput
          title="Coût de production (en €/MWh)"
          error_message="Le coût doit être >= 0"
          min_value="0"
          bind:value={gas_cost}
        />
      </div>
    </div>

    <!-- NUCLEAR PLANT OPTIONS -->
    <div class="collapse collapse-arrow join-item border-base-300 border">
      <input type="checkbox" />
      <div
        class="collapse-title font-semibold flex flex-row items-center gap-1"
      >
        <img
          src={PLANT_ICONS["Nuclear"]}
          alt={PLANT_NAMES["Nuclear"] + "icon"}
          class="inline w-6 h-6"
        />
        {PLANT_NAMES["Nuclear"]}
      </div>
      <div class="collapse-content text-sm">
        <NumericInput
          title="Puissance maximale possible (en MW)"
          error_message="La puissance doit être > 0"
          min_value="1"
          bind:value={nuclear_max_capacity}
        />
        <NumericInput
          title="Coût de production (en €/MWh)"
          error_message="Le coût doit être >= 0"
          min_value="0"
          bind:value={nuclear_cost}
        />
      </div>
    </div>

    <!-- BATTERY OPTIONS -->
    <div class="collapse collapse-arrow join-item border-base-300 border">
      <input type="checkbox" />
      <div
        class="collapse-title font-semibold flex flex-row items-center gap-1"
      >
        <img
          src={PLANT_ICONS["Battery"]}
          alt={PLANT_NAMES["Battery"] + "icon"}
          class="inline w-6 h-6"
        />
        {PLANT_NAMES["Battery"]}
      </div>
      <div class="collapse-content text-sm">
        <NumericInput
          title="Charge maximale possible (en MWh)"
          error_message="La charge doit être > 0"
          min_value="1"
          bind:value={battery_max_charge}
        />
      </div>
    </div>
  </div>
</fieldset>
