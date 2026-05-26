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
    type StackHistory,
    type DeliveryPeriodDetailedScore,
  } from "$lib/message";
  import { PUBLIC_APP_URL } from "$env/static/public";
  import { goto } from "$app/navigation";
  import { SvelteMap } from "svelte/reactivity";
  import Scores from "../../components/organisms/ScoresSummary.svelte";
  import Header from "../../components/molecules/Header.svelte";
  import PlayersReadyList from "../../components/molecules/PlayersReadyList.svelte";
  import FinalScores from "../../components/molecules/FinalScores.svelte";
  import { none, some, type Option } from "$lib/Options";
  import RunningGame from "../../components/pages/game/RunningGame.svelte";

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
  let plant_history: StackHistory = $state(new Map());
  let game_state: GameState = $state("Open");
  let delivery_period_id = $state(0);
  let delivery_period_end: Option<Date> = $state(none());
  let last_delivery_period_id = $state(0);
  let scores: SvelteMap<number, DeliveryPeriodScore> = $state(new SvelteMap());
  let detailed_scores: SvelteMap<number, DeliveryPeriodDetailedScore> = $state(
    new SvelteMap(),
  );
  let final_scores: GameResults = $state(new Array());
  let readiness_status: ReadinessStatus = $state(new SvelteMap());

  const connect = () => {
    // Player and game IDs are stored as cookies and passed when creating the ws connection
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
          const sortedOrderBook = {
            bids: snapshot.bids.toSorted((a, b) => b.price - a.price),
            offers: snapshot.offers.toSorted((a, b) => a.price - b.price),
          };
          orderBook = sortedOrderBook;
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
        .with({ type: "StackHistory" }, ({ history }) => {
          plant_history = history;
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
        .with(
          { type: "DeliveryPeriodResults" },
          ({ delivery_period, score, detailed_score }) => {
            scores.set(delivery_period, score);
            detailed_scores.set(delivery_period, detailed_score);
          },
        )
        .with({ type: "PlayerScores" }, (previous_scores) => {
          for (const [k, v] of previous_scores.scores.entries()) {
            scores.set(Number(k), v);
          }
          for (const [k, v] of previous_scores.detailed_scores.entries()) {
            detailed_scores.set(Number(k), v);
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
  const make_player_ready = () => {
    if (game_state === "Ended") {
      goto("/");
      return;
    }
    sendMessage(JSON.stringify("PlayerIsReady"));
  };
</script>

<main class="h-dvh @container">
  {#if socketIsOpen}
    <div class="flex flex-col items-stretch">
      <Header
        {game_state}
        {player_is_ready}
        {make_player_ready}
        {delivery_period_end}
        periods={{
          current: delivery_period_id,
          last: last_delivery_period_id,
        }}
      />

      <div class="max-w-300 mx-auto w-full pt-3">
        {#if game_state === "Running"}
          <RunningGame
            {orderBook}
            {plant_forecasts}
            {plant_history}
            {plants}
            {sendMessage}
            {trades}
            {trades_to_display}
            {removeTradeToDisplay}
          />
        {:else if game_state === "Open"}
          <PlayersReadyList {player_name} {readiness_status} />
        {:else if game_state === "PostDelivery"}
          <div class="flex flex-col">
            <Scores {detailed_scores} current_period={delivery_period_id} />
          </div>
        {:else if game_state === "Ended"}
          <FinalScores {player_name} {final_scores} />
        {/if}
      </div>
    </div>
  {:else}
    <p>Not connected</p>
  {/if}
</main>
