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

<div class="flex flex-col">
  <h1 class="text-center font-semibold text-xl my-3">Tutoriel</h1>
  <div class="p-4">
    Lorem ipsum dolor sit, amet consectetur adipisicing elit. Dignissimos,
    delectus natus. Vero suscipit ducimus sed incidunt expedita harum, aliquam
    iure? Magnam perferendis in tempore adipisci, cumque natus minima facere
    cupiditate. Lorem ipsum dolor sit, amet consectetur adipisicing elit.
    Suscipit dolores quidem voluptatem voluptates error nihil dolorum, quo rem
    numquam asperiores officiis nisi possimus illo distinctio non, rerum ipsum
    consequatur quos.
  </div>
  <div class="self-center">
    <button onclick={startTutorial} class="text-lg mt-3"
      >➡️ Commencer une partie</button
    >
  </div>
</div>

{#if error}
  <p>Erreur lors de la création du tutoriel</p>
{/if}
