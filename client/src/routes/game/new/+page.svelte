<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";
  import { FORECAST_STEP, generateForecast } from "$lib/forecasts";
  import { PLANT_ICONS, PLANT_NAMES } from "$lib/label";
  import NumericInput from "../../../components/atoms/NumericInput.svelte";

  let game_name = $state("");
  let period_duration_seconds = $state("120");
  let clients_pmax = $state("1800");
  let clients_pmin = $state("300");
  let clients_revenues = $state("56");
  let renewable_pmax = $state("300");
  let gas_installed_capacity = $state("500");
  let gas_cost = $state("80");
  let nuclear_installed_capacity = $state("1000");
  let nuclear_cost = $state("35");
  let battery_charge = $state("300");
  let number_of_periods = $state("4");
  let apiError = $state("");

  let isGameNameValid = $derived(game_name && game_name.trim() !== "");

  let isPeriodDurationValid = $derived(
    period_duration_seconds !== "" &&
      !isNaN(Number(period_duration_seconds)) &&
      Number(period_duration_seconds) >= 30,
  );

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

    return max_power_valid && min_power_valid;
  });

  let consumers_forecasts = $derived(
    generateForecast(
      Number(number_of_periods),
      -Number(clients_pmax),
      -Number(clients_pmin),
    ),
  );
  let renewable_forecasts = $derived(
    generateForecast(Number(number_of_periods), 0, Number(renewable_pmax)),
  );

  let isFormValid = $derived(
    isGameNameValid &&
      isPeriodDurationValid &&
      areGasPlantOptionsValid &&
      areNuclearPlantOptionsValid &&
      areBatteryOptionsValid &&
      isNumberOfPeriodsValid &&
      areConsumersOptionsValid,
  );

  const createGame = async () => {
    const requestBody = {
      game_name: game_name.trim(),
      period_duration_seconds: Number(period_duration_seconds),
      number_of_periods: Number(number_of_periods),
      stack: {
        gas: {
          max_power: Number(gas_installed_capacity),
          cost: Number(gas_cost),
        },
        nuclear: {
          max_power: Number(nuclear_installed_capacity),
          cost: Number(nuclear_cost),
        },
        battery: {
          max_charge: Number(battery_charge),
        },
        consumers: {
          revenues: 56,
          forecasts: consumers_forecasts,
        },
        renewable_forecasts,
      },
    };

    let rest = await fetch(`${PUBLIC_APP_URL}/game`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      mode: "cors",
      credentials: "include",
      body: JSON.stringify(requestBody),
    });
    if (rest.status === 201) {
      let { game_id } = await rest.json();
      goto(`/game/${game_id}/join`);
    } else {
      apiError = "Erreur lors de la création de la partie";
    }
  };
</script>

<div class="flex flex-row justify-center w-full">
  <div class="card w-86 mx-6 mt-6 bg-base-100 shadow-sm">
    <div class="card-body">
      <fieldset class="fieldset bg-base-100">
        <legend class="fieldset-legend text-base">Créer une partie</legend>

        <label class="fieldset-label flex-col"
          ><div class="self-start text-sm">Nom de la partie (requis)</div>
          <input
            type="text"
            class="input validator text-base"
            required
            bind:value={game_name}
          />
        </label>

        <div class="divider divider-start text-sm font-semibold">Options</div>

        <div class="join join-vertical bg-base-100">
          <div class="collapse collapse-arrow join-item border-base-300 border">
            <input type="checkbox" />
            <div class="collapse-title font-semibold">Général</div>
            <div class="collapse-content text-sm">
              <NumericInput
                title="Nombre de périodes"
                error_message="Il doit y avoir au moins une période"
                min_value="1"
                bind:value={number_of_periods}
              />
              <NumericInput
                title="Durée des périodes (en secondes)"
                error_message="La durée doit être d'au moins 30 secondes"
                min_value="30"
                bind:value={period_duration_seconds}
              />
            </div>
          </div>

          <!-- CONSUMERS OPTIONS -->
          <div class="collapse collapse-arrow join-item border-base-300 border">
            <input type="checkbox" />
            <div class="collapse-title font-semibold">
              {PLANT_ICONS["Consumers"]}
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
            <div class="collapse-title font-semibold">
              {PLANT_ICONS["RenewablePlant"]}
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
            <div class="collapse-title font-semibold">
              {PLANT_ICONS["GasPlant"]}
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
            <div class="collapse-title font-semibold">
              {PLANT_ICONS["Nuclear"]}
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
            <div class="collapse-title font-semibold">
              {PLANT_ICONS["Battery"]}
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

        {#if apiError}
          <div class="alert alert-error mt-4 py-2">
            <span>{apiError}</span>
          </div>
        {/if}

        <button
          class="btn btn-neutral mt-4 text-base"
          onclick={createGame}
          disabled={!isFormValid}
        >
          Créer
        </button>
      </fieldset>
    </div>
  </div>
</div>
