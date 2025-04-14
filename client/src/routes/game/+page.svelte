<script lang="ts">
  import { match } from "ts-pattern";
  import {
    parseMessage,
    type DeliveryPeriodScore,
    type GameResults,
    type OrderBook,
    type StackForecasts,
    type StackSnapshot,
    type Trade,
    type ReadinessStatus,
    type GameState,
  } from "$lib/message";
  import { PUBLIC_APP_URL } from "$env/static/public";
  import OrderBookElement from "../../components/organisms/OrderBook.svelte";
  import { goto } from "$app/navigation";
  import Stack from "../../components/organisms/Stack.svelte";
  import { plantsPosition, marketPosition } from "$lib/position";
  import { marketPnl, plantsPnl } from "$lib/pnl";
  import { SvelteMap } from "svelte/reactivity";
  import Scores from "../../components/organisms/ScoresSummary.svelte";
  import Forecasts from "../../components/organisms/Forecasts.svelte";
  import Header from "../../components/molecules/Header.svelte";
  import TradeNotification from "../../components/molecules/TradeNotification.svelte";
  import PlayersReadyList from "../../components/molecules/PlayersReadyList.svelte";
  import FinalScores from "../../components/molecules/FinalScores.svelte";
  import Footer from "../../components/molecules/Footer.svelte";
  import { isSome, none, some, type Option } from "$lib/Options";
  import Countdown from "../../components/atoms/Countdown.svelte";

  let player_name: string = $state("");
  let player_is_ready = $derived.by(() => {
    const player = readiness_status.get(player_name);
    return player === undefined ? false : player;
  });
  let orderBook: OrderBook = $state({
    bids: [],
    offers: [],
  });
  let trades: Trade[] = $state([]);
  let trades_to_display: Trade[] = $state([]);
  const removeTradeToDisplay = (trade_to_remove: Trade) => {
    trades_to_display = trades_to_display.filter(
      (trade) =>
        trade.direction !== trade_to_remove.direction ||
        trade.execution_time !== trade_to_remove.execution_time,
    );
  };
  let plants: StackSnapshot = $state(new Map());
  let plant_forecasts: StackForecasts = $state(new Map());
  let market_state: "Open" | "Closed" = $state("Open");
  let stack_state: "Open" | "Closed" = $state("Open");
  let game_state: GameState = $state("Open");
  let delivery_period_id = $state(0);
  let delivery_period_end: Option<Date> = $state(none());
  let last_delivery_period_id = $state(0);
  let scores: SvelteMap<number, DeliveryPeriodScore> = $state(new SvelteMap());
  let final_scores: GameResults = $state(new Array());
  let readiness_status: ReadinessStatus = $state(new SvelteMap());

  const connect = () => {
    const socket = new WebSocket(`${PUBLIC_APP_URL}/ws`);
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
          trades_to_display.push(new_trade);
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
        .with({ type: "GameState" }, ({ state, delivery_period, end_at }) => {
          game_state = state;
          delivery_period_id = delivery_period;
          if (end_at === "None") {
            delivery_period_end = none();
          } else {
            delivery_period_end = some(new Date(end_at));
          }
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
        .with({ type: "GameResults" }, ({ rankings }) => {
          final_scores = rankings;
        })
        .with({ type: "ReadinessStatus" }, (readiness) => {
          readiness_status = readiness.readiness;
        })
        .with({ type: "YourName" }, (p_name) => {
          player_name = p_name.name;
        })
        .with({ type: "GameDuration" }, ({ last_period }) => {
          last_delivery_period_id = last_period;
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
    if (game_state === "Ended") {
      goto("/");
      return;
    }
    sendMessage(JSON.stringify("PlayerIsReady"));
  };

  let plants_position = $derived(plantsPosition(plants));
  let trades_position = $derived(marketPosition(trades));
  let position = $derived(plants_position + trades_position);

  let plants_pnl = $derived(plantsPnl(plants));
  let market_pnl = $derived(marketPnl(trades));
  let pnl = $derived(plants_pnl + market_pnl);
</script>

<main class="max-w-[600px] mx-auto @container">
  {#if socketIsOpen}
    <div class="flex flex-col gap-6 items-stretch">
      <div
        class={`
        sticky top-0 px-2 pt-5 pb-5
        text-success-content bg-success rounded-b-md
        @sm:p-6
        @min-[600px]:mt-8 @min-[600px]:rounded-t-md
        `}
      >
        <Header
          {game_state}
          {pnl}
          {position}
          periods={{
            current: delivery_period_id,
            last: last_delivery_period_id,
          }}
        />
      </div>

      {#if game_state === "Running"}
        <div class="tabs tabs-lift tabs-md p-3 max-[400px]:p-1">
          <input
            type="radio"
            name="market_forecast_tabs"
            class="tab text-base font-semibold"
            aria-label="Centrales 🔌"
            checked={true}
          />
          <div class="tab-content bg-base-100 border-base-300 p-1 pb-4">
            <Stack {plants} send={sendMessage} />
          </div>
          <input
            type="radio"
            name="market_forecast_tabs"
            class="tab text-base font-semibold"
            aria-label="Marché 💱"
          />
          <div class="tab-content bg-base-100 border-base-300 p-4">
            <OrderBookElement {orderBook} send={sendMessage} {trades} />
          </div>
          <input
            type="radio"
            name="market_forecast_tabs"
            class="tab text-base font-semibold"
            aria-label="Prévisions 🔮"
          />
          <div class="tab-content bg-base-100 border-base-300 p-2">
            <Forecasts {plant_forecasts} plant_snapshots={plants} />
          </div>
        </div>

        <div class="toast mb-15 items-center content-center">
          {#each trades_to_display as trade (`${trade.direction}-${trade.execution_time}`)}
            <TradeNotification {trade} {removeTradeToDisplay} />
          {/each}
        </div>
      {:else if game_state === "Open"}
        <PlayersReadyList {player_name} {readiness_status} />
      {:else if game_state === "PostDelivery"}
        <div class="flex flex-col">
          <Scores {scores} current_period={delivery_period_id} />
        </div>
      {:else if game_state === "Ended"}
        <FinalScores {player_name} {final_scores} />
      {/if}
      {#if isSome(delivery_period_end)}
        <div class="self-center text-lg">
          Termine dans :
          <Countdown end_at={delivery_period_end.value} />
        </div>
      {/if}
      <Footer
        {player_is_ready}
        {game_state}
        {startGame}
        periods={{
          current: delivery_period_id,
          last: last_delivery_period_id,
        }}
      />
    </div>
  {:else}
    <p>Not connected</p>
  {/if}
</main>
