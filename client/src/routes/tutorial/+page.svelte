<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";
  import {
    parseMessage,
    type OrderBook,
    type StackForecasts,
    type StackSnapshot,
    type Trade,
  } from "$lib/message";
  import { match } from "ts-pattern";
  import Intro from "../../components/molecules/tutorial/Intro.svelte";
  import Market from "../../components/molecules/tutorial/Market.svelte";
  import PeriodsAndForecasts from "../../components/molecules/tutorial/PeriodsAndForecasts.svelte";
  import PowerPlants from "../../components/molecules/tutorial/PowerPlants.svelte";
  import { isSome, none, some, type Option } from "$lib/Options";
  import TradeNotification from "../../components/molecules/TradeNotification.svelte";
  import { marketPosition, plantsPosition } from "$lib/position";
  import { marketPnl, plantsPnl } from "$lib/pnl";
  import CurrentScore from "../../components/molecules/CurrentScore.svelte";

  let error = $state(false);
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
  let game_socket: Option<WebSocket> = $state(none());

  const startTutorial = async () => {
    let response = await fetch(`${PUBLIC_APP_URL}/tutorial`, {
      method: "POST",
      mode: "cors",
      credentials: "include",
    });
    if (response.status === 201) {
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
            orderBook.bids = snapshot.bids.toSorted(
              (a, b) => b.price - a.price,
            );
            orderBook.offers = snapshot.offers.toSorted(
              (a, b) => a.price - b.price,
            );
          })
          .with({ type: "NewTrade" }, (new_trade) => {
            trades.push(new_trade);
            trades_to_display.push(new_trade);
          })
          .with({ type: "TradeList" }, (trade_list) => {
            trades = trade_list.trades;
          })
          .with({ type: "StackSnapshot" }, (stack_snapshot) => {
            plants = stack_snapshot.plants;
          })
          .with({ type: "StackForecasts" }, ({ forecasts }) => {
            plant_forecasts = forecasts;
          });
      };
      socket.onopen = () => {
        socket.send(JSON.stringify("ConnectionReady"));
      };
      game_socket = some(socket);
    } else {
      error = true;
    }
  };

  let plants_position = $derived(plantsPosition(plants));
  let trades_position = $derived(marketPosition(trades));
  let position = $derived(plants_position + trades_position);
  let plants_pnl = $derived(plantsPnl(plants));
  let market_pnl = $derived(marketPnl(trades));
  let pnl = $derived(plants_pnl + market_pnl);

  const sendMessage = (msg: string) => {
    if (isSome(game_socket)) {
      game_socket.value.send(msg);
    }
  };
</script>

{#await startTutorial() then}
  <div class="flex flex-col max-w-300 mx-auto text-justify px-3">
    <div class="mt-6">
      <Intro />

      <div class="divider mx-auto max-w-150"></div>

      <div
        class="mx-auto mb-4 p-4 text-success-content bg-success rounded-md sticky top-3 z-30 max-w-200"
      >
        <CurrentScore {position} {pnl} />
      </div>

      <PowerPlants {sendMessage} {plants} />
      <Market {orderBook} {trades} send={sendMessage} />
      <PeriodsAndForecasts forecasts={plant_forecasts} {plants} />
      <div class="my-8 mx-auto">
        <button
          onclick={() => goto("/game")}
          class="btn btn-success text-lg max-w-150 w-full mx-auto block"
          >➡️ Commencer</button
        >
      </div>
    </div>
    <div class="toast mb-15 items-center content-center z-30">
      {#each trades_to_display as trade (`${trade.direction}-${trade.execution_time}`)}
        <TradeNotification {trade} {removeTradeToDisplay} />
      {/each}
    </div>
  </div>

  {#if error}
    <p>Erreur lors de la création du tutoriel</p>
  {/if}
{/await}
