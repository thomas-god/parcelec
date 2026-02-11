<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";

  const startTutorial = async () => {
    let response = await fetch(`${PUBLIC_APP_URL}/tutorial`, {
      method: "POST",
      mode: "cors",
      credentials: "include",
    });
    if (response.status === 201) {
      const socket = new WebSocket(`${PUBLIC_APP_URL}/ws`);

      socket.onopen = () => {
        socket.send(JSON.stringify("ConnectionReady"));
      };
    } else {
      throw new Error("An error occurred while starting the tutorial");
    }

    goto("/game?tutorial=true");
  };
</script>

{#await startTutorial()}
  <div class="mx-auto text-center mt-12">
    <span class="loading loading-spinner loading-xl"></span>
  </div>
{:catch}
  <div
    class="mt-12 mx-12 text-center text-xl font-semibold p-6 bg-error rounded-2xl text-error-content"
  >
    An error occured when starting the tutorial
  </div>
{/await}
