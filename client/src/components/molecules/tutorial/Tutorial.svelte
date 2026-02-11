<script lang="ts">
  import PopupForecasts from "./PopupForecasts.svelte";
  import PopupIntro from "./PopupIntro.svelte";
  import PopupMarket from "./PopupMarket.svelte";
  import PopupPlants from "./PopupPlants.svelte";

  let plantsTutorialPopup = $state(false);
  let marketTutorialPopup = $state(false);
  let forecastsTutorialPopup = $state(false);
</script>

<PopupIntro />
<div class="fab z-100 flex-row-reverse sm:flex-col-reverse">
  <!-- a focusable div with tabindex is necessary to work on all browsers. role="button" is necessary for accessibility -->
  <div tabindex="0" role="button" class="btn btn-lg btn-circle btn-info">
    <img src="/icons/books.svg" alt="Pile of books" class="h-8 w-8" />
  </div>

  <!-- buttons that show up when FAB is open -->
  <button
    class="btn btn-lg btn-circle btn-secondary"
    onclick={() => {
      plantsTutorialPopup = true;
      marketTutorialPopup = false;
      forecastsTutorialPopup = false;
    }}
  >
    <img src="/icons/slider.svg" alt="Slider icon" class="h-8 w-8" /></button
  >
  <button
    class="btn btn-lg btn-circle btn-secondary"
    onclick={() => {
      plantsTutorialPopup = false;
      marketTutorialPopup = true;
      forecastsTutorialPopup = false;
    }}
  >
    <img
      src="/icons/exchange.svg"
      alt="Currency exchange icon"
      class="h-8 w-8"
    /></button
  >
  <button
    class="btn btn-lg btn-circle btn-secondary"
    onclick={() => {
      plantsTutorialPopup = false;
      marketTutorialPopup = false;
      forecastsTutorialPopup = true;
    }}
  >
    <img
      src="/icons/crystal.svg"
      alt="Crystal ball icon"
      class="h-8 w-8"
    /></button
  >
</div>
<PopupPlants bind:open={plantsTutorialPopup} />
<PopupMarket bind:open={marketTutorialPopup} />
<PopupForecasts bind:open={forecastsTutorialPopup} />
