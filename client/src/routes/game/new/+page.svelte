<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";
  import { isNone, isSome, none, type Option } from "$lib/Options";
  import NumericInput from "../../../components/atoms/NumericInput.svelte";
  import CreateFixedStack from "../../../components/organisms/CreateFixedStack.svelte";

  let game_name = $state("");
  let period_duration_seconds = $state("120");
  let number_of_periods = $state("6");
  let fixed_stack_payload: Option<string> = $state(none());
  let apiError = $state("");

  let isGameNameValid = $derived(game_name && game_name.trim() !== "");

  let isPeriodDurationValid = $derived(
    period_duration_seconds !== "" &&
      !isNaN(Number(period_duration_seconds)) &&
      Number(period_duration_seconds) >= 30,
  );

  let isFormValid = $derived(
    isGameNameValid && isPeriodDurationValid && isSome(fixed_stack_payload),
  );

  const createGame = async () => {
    if (isNone(fixed_stack_payload)) {
      return;
    }

    const requestBody = {
      game_name: game_name.trim(),
      period_duration_seconds: Number(period_duration_seconds),
      number_of_periods: Number(number_of_periods),
      stack: fixed_stack_payload.value,
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

          <CreateFixedStack bind:payload={fixed_stack_payload} />

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
        </div>
      </fieldset>
    </div>
  </div>
</div>
