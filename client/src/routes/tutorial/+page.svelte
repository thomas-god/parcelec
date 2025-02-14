<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";

  let error = $state(false);
  const startTutorial = async () => {
    let response = await fetch(`${PUBLIC_APP_URL}/tutorial`, {
      method: "POST",
      mode: "cors",
      credentials: "include",
    });
    if (response.status === 201) {
      goto("/game");
    } else {
      error = true;
    }
  };
</script>

<button onclick={startTutorial}>Commencer</button>

{#if error}
  <p>Erreur lors de la création du tutoriel</p>
{/if}
