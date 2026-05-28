<script lang="ts">
  import { generateForecastValues } from "$lib/forecasts";
  import { PLANT_ICONS, PLANT_NAMES } from "$lib/label";
  import { none, some, type Option } from "$lib/Options";
  import NumericInput from "../atoms/NumericInput.svelte";

  let { payload = $bindable() }: { payload: Option<any> } = $props();

  let clients_pmax = $state("1800");
  let clients_pmin = $state("300");
  let clients_revenues = $state("56");
  let renewable_pmax = $state("300");
  let gas_installed_capacity = $state("500");
  let gas_cost = $state("80");
  let nuclear_installed_capacity = $state("1000");
  let nuclear_cost = $state("35");
  let battery_charge = $state("300");
  let number_of_periods = $state("6");

  let areGasPlantOptionsValid = $derived.by(() => {
    const max_power_valid =
      gas_installed_capacity !== "" &&
      !isNaN(Number(gas_installed_capacity)) &&
      Number(gas_installed_capacity) > 0;

    const cost_valid =
      gas_cost !== "" && !isNaN(Number(gas_cost)) && Number(gas_cost) > 0;

    return max_power_valid && cost_valid;
  });

  let areNuclearPlantOptionsValid = $derived.by(() => {
    const max_power_valid =
      nuclear_installed_capacity !== "" &&
      !isNaN(Number(nuclear_installed_capacity)) &&
      Number(nuclear_installed_capacity) > 0;

    const cost_valid =
      gas_cost !== "" &&
      !isNaN(Number(nuclear_cost)) &&
      Number(nuclear_cost) > 0;

    return max_power_valid && cost_valid;
  });

  let areBatteryOptionsValid = $derived.by(() => {
    const max_charge_valid =
      battery_charge !== "" &&
      !isNaN(Number(battery_charge)) &&
      Number(battery_charge) > 0;

    return max_charge_valid;
  });

  let isNumberOfPeriodsValid = $derived(
    number_of_periods !== "" &&
      !isNaN(Number(number_of_periods)) &&
      Number(number_of_periods) > 0,
  );

  let areConsumersOptionsValid = $derived.by(() => {
    const max_power_valid =
      clients_pmax !== "" &&
      !isNaN(Number(clients_pmax)) &&
      Number(clients_pmax) > 0;

    const min_power_valid =
      clients_pmin !== "" &&
      !isNaN(Number(clients_pmin)) &&
      Number(clients_pmin) > 0 &&
      Number(clients_pmin) <= Number(clients_pmax);

    const revenues_valid =
      clients_revenues !== "" &&
      !isNaN(Number(clients_revenues)) &&
      Number(clients_revenues) > 0;

    return max_power_valid && min_power_valid && revenues_valid;
  });

  let consumers_forecasts = $derived(
    generateForecastValues(
      Number(number_of_periods),
      -Number(clients_pmax),
      -Number(clients_pmin),
    ),
  );
  let renewable_forecasts = $derived(
    generateForecastValues(
      Number(number_of_periods),
      0,
      Number(renewable_pmax),
    ),
  );

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
          gas_capacity: Number(gas_installed_capacity),
          nuclear_cost: Number(nuclear_cost),
          nuclear_capacity: Number(nuclear_installed_capacity),
          battery_capacity: Number(battery_charge),
          consumers_revenues: Number(clients_revenues),
          consumers_forecasts,
          consumers_forecasts_range: Number(number_of_periods),
          renewable_forecasts,
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
          bind:value={clients_pmax}
        />
        <label class="fieldset-label flex-col">
          <div class="flex justify-between items-center w-full">
            <div class="self-start text-sm">
              Puissance minimale consommée par les clients (en MW)
            </div>
          </div>
          <input
            type="number"
            min="1"
            max={clients_pmax}
            class="input validator text-base"
            bind:value={clients_pmin}
            required
          />
          <p class="validator-hint self-start mt-0">
            La puissance doit être > 0 et inférieure à Pmax
          </p>
        </label>
        <NumericInput
          title="Revenus (en €/MWh)"
          error_message="Le revenue doit être >= 0"
          min_value="0"
          bind:value={clients_revenues}
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
          title="Puissance installée (en MW)"
          error_message="La puissance doit être > 0"
          min_value="1"
          bind:value={renewable_pmax}
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
          title="Puissance installée (en MW)"
          error_message="La puissance doit être > 0"
          min_value="1"
          bind:value={gas_installed_capacity}
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
          title="Puissance installée (en MW)"
          error_message="La puissance doit être > 0"
          min_value="1"
          bind:value={nuclear_installed_capacity}
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
          title="Charge maximale (en MWh)"
          error_message="La charge doit être > 0"
          min_value="1"
          bind:value={battery_charge}
        />
      </div>
    </div>
  </div>
</fieldset>
