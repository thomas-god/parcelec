<script lang="ts">
  import { match } from "ts-pattern";
  import {
    parseMessage,
    type DeliveryPeriodScore,
    type GameResults,
    type MarketForecast,
    type OrderBook,
    type StackForecasts,
    type StackSnapshot,
    type Trade,
    type ReadinessStatus,
  } from "$lib/message";
  import { PUBLIC_WS_URL } from "$env/static/public";
  import OrderBookElement from "../../components/organisms/OrderBook.svelte";
  import { goto } from "$app/navigation";
  import Stack from "../../components/organisms/Stack.svelte";
  import CurrentScore from "../../components/molecules/CurrentScore.svelte";
  import { plantsPosition, marketPosition } from "$lib/position";
  import { marketPnl, plantsPnl } from "$lib/pnl";
  import { SvelteMap } from "svelte/reactivity";
  import Scores from "../../components/organisms/ScoresSummary.svelte";
  import Forecasts from "../../components/organisms/Forecasts.svelte";

  let player_name: String = $state("");
  let player_is_ready = $derived.by(() => {
    let found = sorted_readines_status.find(
      (status) => status.player === player_name,
    );
    if (found === undefined) {
      return false;
    }
    return found.ready;
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
  let game_state: "Open" | "Running" | "PostDelivery" | "Ended" =
    $state("Open");
  let delivery_period_id = $state(0);
  let scores: SvelteMap<number, DeliveryPeriodScore> = $state(new SvelteMap());
  let final_scores: GameResults = $state(new Array());
  let market_forecasts: SvelteMap<number, MarketForecast[]> = $state(
    new SvelteMap(),
  );
  let readiness_status: ReadinessStatus = $state(new SvelteMap());
  let sorted_readines_status = $derived.by(() => {
    let sorted_status: { player: string; ready: boolean }[] = [];
    for (const [player, ready] of readiness_status) {
      sorted_status.push({ player, ready });
    }
    sorted_status.sort((a, b) => (a.player > b.player ? 1 : -1));
    return sorted_status;
  });
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
        .with({ type: "GameResults" }, ({ rankings }) => {
          final_scores = rankings;
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
        .with({ type: "ReadinessStatus" }, (readiness) => {
          readiness_status = readiness.readiness;
        })
        .with({ type: "YourName" }, (p_name) => {
          player_name = p_name.name;
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

<main class="max-w-[600px] mx-auto">
  {#if socketIsOpen}
    <div class="flex flex-col gap-6 items-stretch">
      <div
        class="sticky top-0 px-2 py-5 @sm:p-6 text-success-content bg-success rounded-b-md"
      >
        {#if game_state === "Running"}
          <CurrentScore {position} {pnl} />
        {:else if game_state === "Open"}
          <div class="text-2xl text-center mx-auto">
            En attente d'autres joueurs
          </div>
        {:else if game_state === "Ended"}
          <div class="text-2xl text-center mx-auto">Partie terminée !</div>
        {:else}
          <div class="text-2xl text-center mx-auto">Phase terminée !</div>
        {/if}
      </div>

      {#if game_state === "Running"}
        <div class="tabs tabs-border tabs-sm p-1">
          <input
            type="radio"
            name="market_forecast_tabs"
            class="tab text-base font-semibold"
            aria-label="Centrales 🔌"
            checked={true}
          />
          <div class="tab-content bg-base-100 border-base-300 p-4">
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
          <div class="tab-content bg-base-100 border-base-300 p-4">
            <Forecasts
              market_forecasts={market_forecasts.get(delivery_period_id + 1)!}
              {plant_forecasts}
              plant_snapshots={plants}
            />
          </div>
        </div>

        <div class="toast mb-14 items-center content-center">
          {#each trades_to_display as trade (`${trade.direction}-${trade.execution_time}`)}
            <div class="alert alert-info self-center">
              <span
                >Nouveau trade: {trade.volume}MW {trade.direction === "Buy"
                  ? "achetés"
                  : "vendus"}
                @ {0.01 * (trade.price as number)}€
                <button
                  class="text-lg pl-2"
                  onclick={() => removeTradeToDisplay(trade)}>✖️</button
                >
              </span>
            </div>
          {/each}
        </div>
      {:else if game_state === "Open"}
        <ul class="list bg-base-100 rounded-box shadow-md">
          <li class="p-4 pb-2 text-xs opacity-60 tracking-wide">Joueurs</li>
          {#each sorted_readines_status as { player, ready } (player)}
            <li class="list-row">
              {#if player === player_name}
                <div class="list-col-grow font-semibold">
                  {player}
                </div>
              {:else}
                <div class="list-col-grow">{player}</div>
              {/if}
              <div>
                {ready ? "✅" : "⌛"}
              </div>
            </li>
          {/each}
        </ul>
      {:else if game_state === "PostDelivery"}
        <div class="flex flex-col">
          <Scores {scores} current_period={delivery_period_id} />
        </div>
      {:else if game_state === "Ended"}
        <ol class="list bg-base-100 rounded-box shadow-md">
          {#each final_scores as score (score.player)}
            <li class="list-row items-center">
              <div class="text-4xl font-thin opacity-30 tabular-nums">
                {score.rank}
              </div>
              {#if score.player === player_name}
                <div class="font-semibold">
                  {score.player}
                </div>
              {:else}
                <div>
                  {score.player}
                </div>
              {/if}
              <div>
                {score.score.toLocaleString("fr-FR", {
                  signDisplay: "exceptZero",
                })} €
                {#if score.tier === "Gold"}
                  <!-- 🥇 -->
                  ⭐⭐⭐
                {:else if score.tier === "Silver"}
                  <!-- 🥈 -->
                  ⭐⭐
                {:else if score.tier === "Bronze"}
                  <!-- 🥉 -->
                  ⭐
                {:else}
                  👍
                {/if}
              </div>
            </li>
          {/each}
        </ol>
      {/if}
      <footer
        class="footer fixed bottom-0 bg-success text-success-content rounded-t-md p-2 pb-4 w-screen max-w-[600px] flex flex-col items-center text-xl"
      >
        <button onclick={startGame}>
          {#if game_state === "Running"}
            {#if player_is_ready}
              En attente des autres joueurs
            {:else}
              Terminer la phase ➡️
            {/if}
          {:else if game_state === "Open"}
            {#if player_is_ready}
              En attente des autres joueurs
            {:else}
              Commencer la partie ➡️
            {/if}
          {:else if game_state === "Ended"}
            Retour au menu ➡️
          {:else if player_is_ready}
            En attente des autres joueurs
          {:else}
            Phase suivante ➡️
          {/if}</button
        >
      </footer>
    </div>
  {:else}
    <p>Not connected</p>
  {/if}
</main>
