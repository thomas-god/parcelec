<script lang="ts">
  import OrderBookElement from "../../organisms/OrderBook.svelte";
  import Stack from "../../organisms/Stack.svelte";
  import Forecasts from "../../organisms/Forecasts.svelte";
  import TradeNotification from "../../molecules/TradeNotification.svelte";
  import Portfolio from "../../organisms/Portfolio.svelte";
  import { page } from "$app/state";
  import Tutorial from "../../molecules/tutorial/Tutorial.svelte";
  import type {
    OrderBook,
    StackForecasts,
    StackHistory,
    StackSnapshot,
    Trade,
  } from "$lib/message";

  interface Props {
    plants: StackSnapshot;
    plant_forecasts: StackForecasts;
    plant_history: StackHistory;
    trades: Trade[];
    trades_to_display: Trade[];
    removeTradeToDisplay: (trade_to_remove: Trade) => void;
    orderBook: OrderBook;
    sendMessage: (msg: string) => void;
  }

  let {
    plants,
    trades,
    trades_to_display,
    removeTradeToDisplay,
    sendMessage,
    orderBook,
    plant_forecasts,
    plant_history,
  }: Props = $props();

  let isTutorial = page.url.searchParams.has("tutorial", "true");
</script>

<div class="flex flex-col gap-3 items-stretch">
  <div class="portfolio-wrapper">
    <div class="bg-base-100 border border-base-300 rounded-lg px-3 pb-1">
      <Portfolio {plants} {trades} />
    </div>
  </div>

  <!-- Mobile: Tabs layout -->
  <div class="mobile-tabs-layout tabs tabs-lift tabs-md">
    <label class="tab text-base font-semibold">
      <input
        type="radio"
        name="market_forecast_tabs"
        class="tab text-base font-semibold"
        checked={true}
      />
      Pilotage
      <img
        src="/icons/slider.svg"
        alt="Slider icon"
        class="ml-1 w-5 h-5 inline"
      />
    </label>
    <div class="tab-content bg-base-100 border-base-300 p-1 pb-4">
      <Stack {plants} send={sendMessage} />
    </div>
    <label class="tab text-base font-semibold">
      <input
        type="radio"
        name="market_forecast_tabs"
        class="tab text-base font-semibold"
      />
      Marché
      <img
        src="/icons/exchange.svg"
        alt="Exchange icon"
        class="ml-1 w-5 h-5 inline"
      />
    </label>
    <div class="tab-content bg-base-100 border-base-300 p-2 pt-4">
      <OrderBookElement {orderBook} send={sendMessage} {trades} />
    </div>
    <label class="tab text-base font-semibold">
      <input
        type="radio"
        name="market_forecast_tabs"
        class="tab text-base font-semibold"
        aria-label="Prévisions 🔮"
      />
      Prévisions
      <img
        src="/icons/crystal.svg"
        alt="Crystal ball icon"
        class="ml-1 w-5 h-5 inline"
      />
    </label>
    <div class="tab-content bg-base-100 border-base-300 p-2">
      <Forecasts
        {plant_forecasts}
        plant_snapshots={plants}
        history={plant_history}
      />
    </div>
  </div>

  <!-- Desktop: Side-by-side layout -->
  <div class="desktop-grid-layout">
    <div class="bg-base-100 border border-base-300 rounded-lg p-2">
      <h3
        class="text-lg text-center font-semibold pt-2 pb-4 flex flex-row items-center justify-center gap-1"
      >
        Pilotage <img
          src="/icons/slider.svg"
          alt="Slider icon"
          class="w-8 h-8 inline"
        />
      </h3>
      <Stack {plants} send={sendMessage} />
    </div>
    <div class="bg-base-100 border border-base-300 rounded-lg p-2">
      <h3
        class="text-lg text-center font-semibold pt-2 pb-4 flex flex-row items-center justify-center gap-1"
      >
        Marché <img
          src="/icons/exchange.svg"
          alt="Currency exchange icon"
          class="w-8 h-8 inline"
        />
      </h3>
      <OrderBookElement {orderBook} send={sendMessage} {trades} />
    </div>
    <div class="bg-base-100 border border-base-300 rounded-lg p-2">
      <h3
        class="text-lg text-center align-middle font-semibold pt-2 flex flex-row items-center justify-center gap-1"
      >
        Prévisions
        <img
          src="/icons/crystal.svg"
          alt="Crystal ball icon"
          class="w-8 h-8 inline"
        />
      </h3>
      <Forecasts
        {plant_forecasts}
        plant_snapshots={plants}
        history={plant_history}
      />
    </div>
  </div>

  <div class="toast mb-15 items-center content-center">
    {#each trades_to_display as trade (`${trade.direction}-${trade.execution_time}`)}
      <TradeNotification {trade} {removeTradeToDisplay} />
    {/each}
  </div>

  {#if isTutorial}
    <Tutorial />
  {/if}
</div>

<style>
  .portfolio-wrapper {
    padding-left: 0.75rem;
    padding-right: 0.75rem;
    padding-top: 1rem;

    @media (max-width: 400px) {
      padding-left: calc(var(--spacing) * 1);
      padding-right: calc(var(--spacing) * 1);
      padding-top: calc(var(--spacing) * 1);
    }
    @container (min-width: 1200px) {
      padding-left: 1rem;
      padding-right: 1rem;
    }
  }

  .mobile-tabs-layout {
    padding-left: calc(var(--spacing) * 3);
    padding-right: calc(var(--spacing) * 3);

    @media (max-width: 400px) {
      padding: calc(var(--spacing) * 1);
    }

    @container (min-width: 1200px) {
      display: none;
    }
  }

  .desktop-grid-layout {
    display: none;

    @container (min-width: 1200px) {
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: calc(var(--spacing) * 4);
      padding-left: calc(var(--spacing) * 4);
      padding-right: calc(var(--spacing) * 4);
    }
  }
</style>
