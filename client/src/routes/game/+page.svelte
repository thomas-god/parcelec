<script lang="ts">
  import { match } from "ts-pattern";
  import {
    parseMessage,
    type DeliveryPeriodScore,
    type MarketForecast,
    type OrderBook,
    type StackForecasts,
    type StackSnapshot,
    type Trade,
  } from "$lib/message";
  import { PUBLIC_WS_URL } from "$env/static/public";
  import OrderBookElement from "../../components/organisms/OrderBook.svelte";
  import { goto } from "$app/navigation";
  import Stack from "../../components/organisms/StackSliders.svelte";
  import LastScore from "../../components/molecules/LastScore.svelte";
  import { fade } from "svelte/transition";
  import { plantsPosition, marketPosition } from "$lib/position";
  import { marketPnl, plantsPnl } from "$lib/pnl";
  import { SvelteMap } from "svelte/reactivity";
  import Scores from "../../components/organisms/Scores.svelte";
  import Forecasts from "../../components/organisms/Forecasts.svelte";

  let orderBook: OrderBook = $state({
    bids: [],
    offers: [],
  });
  let trades: Trade[] = $state([]);
  let plants: StackSnapshot = $state(new Map());
  let plant_forecasts: StackForecasts = $state(new Map());
  let market_state: "Open" | "Closed" = $state("Open");
  let stack_state: "Open" | "Closed" = $state("Open");
  let game_state: "Open" | "Running" | "PostDelivery" = $state("Open");
  let delivery_period_id = $state(0);
  let scores: SvelteMap<number, DeliveryPeriodScore> = $state(new SvelteMap());
  let market_forecasts: SvelteMap<number, MarketForecast[]> = $state(
    new SvelteMap(),
  );

  const connect = () => {
    const socket = new WebSocket(`${PUBLIC_WS_URL}/ws`);
    socket.onmessage = (msg) => {
      const parseRes = parseMessage(msg.data);
      if (!parseRes.success) {
        console.log(
          `Error while parsing message ${msg.data}: ${parseRes.error}`,
        );
        return;
      }

      match(parseRes.data)
        .with({ type: "OrderBookSnapshot" }, (snapshot) => {
          orderBook.bids = snapshot.bids.toSorted((a, b) => b.price - a.price);
          orderBook.offers = snapshot.offers.toSorted(
            (a, b) => a.price - b.price,
          );
        })
        .with({ type: "NewTrade" }, (new_trade) => {
          trades.push(new_trade);
          show_last_trade = true;
          debouncedHideLastTrade();
        })
        .with({ type: "StackSnapshot" }, (stack_snapshot) => {
          plants = stack_snapshot.plants;
        })
        .with({ type: "StackForecasts" }, ({ forecasts }) => {
          plant_forecasts = forecasts;
        })
        .with({ type: "TradeList" }, (trade_list) => {
          trades = trade_list.trades;
        })
        .with({ type: "GameState" }, ({ state, delivery_period }) => {
          game_state = state;
          delivery_period_id = delivery_period;
          if (state === "Running") {
            console.log(`Starting delivery period no: ${delivery_period_id}`);
          }
        })
        .with({ type: "MarketState" }, ({ state }) => {
          market_state = state;
        })
        .with({ type: "StackState" }, ({ state }) => {
          stack_state = state;
        })
        .with(
          { type: "DeliveryPeriodResults" },
          ({ delivery_period, score }) => {
            scores.set(delivery_period, score);
          },
        )
        .with({ type: "PlayerScores" }, (previous_scores) => {
          for (const [k, v] of previous_scores.scores.entries()) {
            scores.set(Number(k), v);
          }
        })
        .with({ type: "NewMarketForecast" }, (forecast) => {
          if (market_forecasts.has(forecast.period)) {
            market_forecasts.get(forecast.period)!.push(forecast);
          } else {
            market_forecasts.set(forecast.period, [forecast]);
          }
        })
        .with({ type: "MarketForecasts" }, (forecasts) => {
          for (const forecast of forecasts.forecasts) {
            if (market_forecasts.has(forecast.period)) {
              market_forecasts.get(forecast.period)!.push(forecast);
            } else {
              market_forecasts.set(forecast.period, [forecast]);
            }
          }
        })
        .exhaustive();
    };
    socket.onopen = () => {
      socket.send(JSON.stringify("ConnectionReady"));
      socketIsOpen = true;
    };
    socket.onclose = () => {
      socketIsOpen = false;
      goto("/");
    };
    return socket;
  };

  let socket = connect();
  let socketIsOpen = $state(false);

  const sendMessage = (msg: string) => {
    socket.send(msg);
  };
  const startGame = () => {
    sendMessage(JSON.stringify("PlayerIsReady"));
  };
  let show_last_trade = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout>;
  const debouncedHideLastTrade = () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      show_last_trade = false;
    }, 3000);
  };
  let plants_position = $derived(plantsPosition(plants));
  let trades_position = $derived(marketPosition(trades));
  let position = $derived(plants_position + trades_position);

  let plants_pnl = $derived(plantsPnl(plants));
  let market_pnl = $derived(marketPnl(trades));
  let pnl = $derived(plants_pnl + market_pnl);
</script>

<main class="max-w-[600px] mx-auto">
  {#if socketIsOpen}
    <div class="flex flex-col gap-6 items-stretch">
      <div
        class="sticky top-0 px-2 py-5 @sm:p-6 text-success-content bg-success rounded-b-md"
      >
        {#if game_state === "Running"}
          <LastScore {position} {pnl} />
        {:else}
          <div class="text-2xl text-center mx-auto">Phase terminée !</div>
        {/if}
      </div>

      {#if game_state === "Running"}
        <div class="tabs tabs-lift tabs-sm p-1">
          <input
            type="radio"
            name="market_forecast_tabs"
            class="tab text-base font-semibold"
            aria-label="Centrales 🔌"
            checked={true}
          />
          <div class="tab-content bg-base-100 border-base-300 p-6">
            <Stack {plants} send={sendMessage} />
          </div>
          <input
            type="radio"
            name="market_forecast_tabs"
            class="tab text-base font-semibold"
            aria-label="Marché 💱"
            checked={true}
          />
          <div class="tab-content bg-base-100 border-base-300 p-6">
            <OrderBookElement {orderBook} send={sendMessage} {trades} />
          </div>
          <input
            type="radio"
            name="market_forecast_tabs"
            class="tab text-base font-semibold"
            aria-label="Prévisions 🔮"
          />
          <div class="tab-content bg-base-100 border-base-300 p-6">
            <Forecasts
              market_forecasts={market_forecasts.get(delivery_period_id + 1)!}
              {plant_forecasts}
            />
          </div>
        </div>
        {#if show_last_trade && trades.length > 0}
          <div
            transition:fade
            id="bottom-banner"
            tabindex="-1"
            class="fixed bottom-0 start-0 z-50 flex justify-between w-full p-4 border-t border-gray-200 bg-gray-50 dark:bg-gray-700 dark:border-gray-600"
          >
            <div class="flex items-center mx-auto">
              <p
                class="flex items-center text-md font-normal text-gray-500 dark:text-gray-400"
              >
                <span
                  >Nouveau trade: {trades.at(-1)?.volume}MW {trades.at(-1)
                    ?.direction === "Buy"
                    ? "achetés"
                    : "vendus"}
                  @ {0.01 * (trades.at(-1)?.price as number)}€ 🤑</span
                >
              </p>
            </div>
          </div>
        {/if}
        <!-- </div> -->
      {:else if game_state === "Open"}
        <p>En attente d'autres joueurs</p>
        <button onclick={startGame}>Ready!</button>
      {:else if game_state === "PostDelivery"}
        <div class="flex flex-col">
          <Scores {scores} current_period={delivery_period_id} />
          <!-- {#if scores.has(delivery_period_id)}
            <div class="mt-8 self-center text-lg">
              <ul>
                <li>
                  Equilibre: {scores
                    .get(delivery_period_id)!
                    .balance.toLocaleString("fr-FR")} MW
                </li>
                <li>
                  PnL: {scores
                    .get(delivery_period_id)!
                    .pnl.toLocaleString("fr-FR")}
                  €
                </li>
                <li>
                  Ecarts: {scores
                    .get(delivery_period_id)!
                    .imbalance_cost.toLocaleString("fr-FR")} €
                </li>
                <li class="font-semibold">
                  Total: {scores.get(delivery_period_id)!.pnl +
                    scores.get(delivery_period_id)!.imbalance_cost} €
                </li>
              </ul>
            </div>
          {/if} -->
        </div>
      {/if}
      <div
        class="fixed bottom-0 bg-success text-success-content rounded-t-md p-2 pb-4 w-screen max-w-[600px] flex flex-col items-center text-xl"
      >
        <button onclick={startGame}>
          {#if game_state === "Running"}
            Terminer la phase ➡️
          {:else}
            Phase suivante ➡️
          {/if}</button
        >
      </div>
    </div>
  {:else}
    <p>Not connected</p>
  {/if}
</main>
