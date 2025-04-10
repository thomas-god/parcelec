<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";

  let game_name = $state("");
  let period_duration_seconds = $state("120");
  let apiError = $state("");

  let isGameNameValid = $derived(game_name && game_name.trim() !== "");

  let isPeriodDurationValid = $derived(
    period_duration_seconds !== "" &&
      !isNaN(Number(period_duration_seconds)) &&
      Number(period_duration_seconds) >= 30,
  );

  let isFormValid = $derived(isGameNameValid && isPeriodDurationValid);

  let gameNameError = $derived(
    !isGameNameValid && game_name !== ""
      ? "Le nom de la partie est requis"
      : "",
  );

  let periodDurationError = $derived(
    period_duration_seconds !== "" &&
      !isNaN(Number(period_duration_seconds)) &&
      Number(period_duration_seconds) < 30
      ? "La durée doit être d'au moins 30 secondes"
      : "",
  );

  const createGame = async () => {
    const requestBody = {
      game_name: game_name.trim(),
      period_duration_seconds: Number(period_duration_seconds),
      stack: {
        gas: {
          max_power: 500,
          cost: 80,
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
            class="input text-base"
            class:input-error={gameNameError}
            bind:value={game_name}
            onkeypress={(key) => {
              if (key.code === "Enter") {
                createGame();
              }
            }}
          />
          {#if gameNameError}
            <div class="text-error text-xs mt-1">{gameNameError}</div>
          {/if}
        </label>

        <label class="fieldset-label flex-col mt-4">
          <div class="flex justify-between items-center w-full">
            <div class="self-start text-sm">
              Durée des périodes (en secondes)
            </div>
          </div>
          <input
            type="number"
            min="30"
            class="input text-base"
            class:input-error={periodDurationError}
            bind:value={period_duration_seconds}
            placeholder="120"
          />
          {#if periodDurationError}
            <div class="text-error text-xs mt-1">{periodDurationError}</div>
          {/if}
        </label>

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
