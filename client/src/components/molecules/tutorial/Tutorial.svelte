<script lang="ts">
  import { SHOW_TUTORIAL_INTRO_POPUP } from "$lib/storage/keys";
  import PopupForecasts from "./PopupForecasts.svelte";
  import PopupIntro from "./PopupIntro.svelte";
  import PopupMarket from "./PopupMarket.svelte";
  import PopupPlants from "./PopupPlants.svelte";

  let introPopup = $state(
    (sessionStorage.getItem(SHOW_TUTORIAL_INTRO_POPUP) || "true") === "true",
  );
  let plantsPopup = $state(false);
  let marketPopup = $state(false);
  let forecastsPopup = $state(false);
</script>

<PopupIntro
  bind:open={
    () => introPopup,
    (v) => {
      introPopup = v;
      sessionStorage.setItem(SHOW_TUTORIAL_INTRO_POPUP, v ? "true" : "false");
    }
  }
/>
<div class="fab z-100 flex-row-reverse sm:flex-col-reverse">
  <!-- a focusable div with tabindex is necessary to work on all browsers. role="button" is necessary for accessibility -->
  <div tabindex="0" role="button" class="btn btn-lg btn-circle btn-info">
    <img
      src="/icons/question-mark.svg"
      alt="Question mark icon"
      class="h-8 w-8"
    />
  </div>

  <!-- buttons that show up when FAB is open -->
  <button
    class="btn btn-lg btn-circle btn-secondary"
    onclick={() => {
      introPopup = true;
      plantsPopup = false;
      marketPopup = false;
      forecastsPopup = false;
    }}
  >
    <img src="/icons/books.svg" alt="Slider icon" class="h-8 w-8" /></button
  >
  <button
    class="btn btn-lg btn-circle btn-secondary"
    onclick={() => {
      introPopup = false;
      plantsPopup = true;
      marketPopup = false;
      forecastsPopup = false;
    }}
  >
    <img src="/icons/slider.svg" alt="Slider icon" class="h-8 w-8" /></button
  >
  <button
    class="btn btn-lg btn-circle btn-secondary"
    onclick={() => {
      introPopup = false;
      plantsPopup = false;
      marketPopup = true;
      forecastsPopup = false;
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
      introPopup = false;
      plantsPopup = false;
      marketPopup = false;
      forecastsPopup = true;
    }}
  >
    <img
      src="/icons/crystal.svg"
      alt="Crystal ball icon"
      class="h-8 w-8"
    /></button
  >
</div>
<PopupPlants bind:open={plantsPopup} />
<PopupMarket bind:open={marketPopup} />
<PopupForecasts bind:open={forecastsPopup} />
