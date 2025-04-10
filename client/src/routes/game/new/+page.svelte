<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";
  import { PLANT_ICONS, PLANT_NAMES } from "$lib/label";
  import NumericInput from "../../../components/atoms/NumericInput.svelte";

  let game_name = $state("");
  let period_duration_seconds = $state("120");
  let gas_installed_capacity = $state("500");
  let gas_cost = $state("80");
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

    return max_power_valid && gas_cost;
  });

  let isFormValid = $derived(
    isGameNameValid && isPeriodDurationValid && areGasPlantOptionsValid,
  );

  const createGame = async () => {
    const requestBody = {
      game_name: game_name.trim(),
      period_duration_seconds: Number(period_duration_seconds),
      stack: {
        gas: {
          max_power: gas_installed_capacity,
          cost: gas_cost,
        },
        nuclear: {
          max_power: 1000,
          cost: 35,
        },
        battery: {
          max_charge: 300,
        },
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

        <div class="divider divider-start">Options</div>

        <NumericInput
          title="Durée des périodes (en secondes)"
          error_message="La durée doit être d'au moins 30 secondes"
          min_value="30"
          bind:value={period_duration_seconds}
        />

        <!-- GAS PLANT OPTIONS -->
        <div class="divider divider-start">
          {PLANT_ICONS["GasPlant"]}
          {PLANT_NAMES["GasPlant"]}
        </div>
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

        <!-- <label class="fieldset-label flex-col">
          <div class="flex justify-between items-center w-full">
            <div class="self-start text-sm">
              Durée des périodes (en secondes)
            </div>
          </div>
          <input
            type="number"
            min="30"
            class="input validator text-base"
            bind:value={period_duration_seconds}
            required
          />
          <p class="validator-hint self-start mt-0">
            La durée doit être d'au moins 30 secondes
          </p>
        </label> -->

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
