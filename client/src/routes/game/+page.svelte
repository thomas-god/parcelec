<script lang="ts">
  import { match } from "ts-pattern";
  import {
    parseMessage,
    type DeliveryPeriodScore,
    type OrderBook,
    type StackSnapshot,
    type Trade,
  } from "$lib/message";
  import { PUBLIC_WS_URL } from "$env/static/public";
  import OrderBookElement from "../../components/organisms/OrderBook.svelte";
  import { goto } from "$app/navigation";
  import Stack from "../../components/organisms/StackSliders.svelte";
  import Scores from "../../components/molecules/Scores.svelte";
  import { fade } from "svelte/transition";
  import { isSome, none, some, unwrap, type Option } from "$lib/Options";
  import { plantsPosition, marketPosition } from "$lib/position";
  import { marketPnl, plantsPnl } from "$lib/pnl";

  let orderBook: OrderBook = $state({
    bids: [],
    offers: [],
  });
  let trades: Trade[] = $state([]);
  let plants: StackSnapshot = $state(new Map());
  let market_state: "Open" | "Closed" = $state("Open");
  let stack_state: "Open" | "Closed" = $state("Open");
  let game_state: "Open" | "Running" | "PostDelivery" = $state("Open");
  let post_delivery_score: Option<DeliveryPeriodScore> = $state(none());

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
        .with({ type: "TradeList" }, (trade_list) => {
          trades = trade_list.trades;
        })
        .with({ type: "GameState" }, ({ state }) => {
          game_state = state;
        })
        .with({ type: "MarketState" }, ({ state }) => {
          market_state = state;
        })
        .with({ type: "StackState" }, ({ state }) => {
          stack_state = state;
        })
        .with({ type: "DeliveryPeriodResults" }, (score) => {
          post_delivery_score = some(score);
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
          <Scores {position} {pnl} />
        {:else}
          <div class="text-2xl text-center mx-auto">Phase terminée !</div>
        {/if}
      </div>

      {#if game_state === "Running"}
        <Stack {plants} send={sendMessage} />
        <OrderBookElement {orderBook} send={sendMessage} {trades} />
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
          <!-- <h2 class="text-center font-semibold text-xl mt-6">
            Période de livraison terminée !
          </h2> -->
          {#if isSome(post_delivery_score)}
            <div class="mt-8 self-center text-lg">
              <ul>
                <li>
                  Equilibre: {unwrap(
                    post_delivery_score,
                  ).balance.toLocaleString("fr-FR")} MW
                </li>
                <li>
                  PnL: {unwrap(post_delivery_score).pnl.toLocaleString("fr-FR")}
                  €
                </li>
                <li>
                  Ecarts: {unwrap(
                    post_delivery_score,
                  ).imbalance_cost.toLocaleString("fr-FR")} €
                </li>
                <li class="font-semibold">
                  Total: {unwrap(post_delivery_score).pnl +
                    unwrap(post_delivery_score).imbalance_cost} €
                </li>
              </ul>
            </div>
          {/if}
          <!-- <div class="mt-8 self-center text-lg">
            <button onclick={startGame}> ➡️ Partie suivante</button>
          </div> -->
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
