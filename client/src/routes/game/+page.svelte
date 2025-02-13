<script lang="ts">
  import { match } from "ts-pattern";
  import {
    parseMessage,
    type OrderBook,
    type StackSnapshot,
    type Trade,
  } from "$lib/message";
  import { PUBLIC_WS_URL } from "$env/static/public";
  import OrderBookElement from "../../components/organisms/OrderBook.svelte";
  import { goto } from "$app/navigation";
  import Stack from "../../components/organisms/Stack.svelte";
  import Scores from "../../components/molecules/Scores.svelte";
  import { fade } from "svelte/transition";

  let orderBook: OrderBook = $state({
    bids: [],
    offers: [],
  });
  let trades: Trade[] = $state([]);
  let plants: StackSnapshot = $state(new Map());
  let market_state: "Open" | "Closed" = $state("Open");
  let stack_state: "Open" | "Closed" = $state("Open");

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
          const trades_to_add = [];
          for (const trade of trade_list.trades) {
            if (!trades.includes(trade)) {
              trades_to_add.push(trade);
            }
          }
          trades = trades.concat(trades_to_add);
        })
        .with({ type: "MarketState" }, ({ state }) => {
          market_state = state;
        })
        .with({ type: "StackState" }, ({ state }) => {
          stack_state = state;
        })
        .exhaustive();
    };
    socket.onopen = () => {
      socket.send(JSON.stringify("ConnectionReady"));
      socketIsOpen = true;
    };
    socket.onclose = () => {
      socketIsOpen = false;
      goto("/game/join");
    };
    return socket;
  };

  let socket = connect();
  let socketIsOpen = $state(false);

  const sendMessage = (msg: string) => {
    socket.send(msg);
  };

  let show_last_trade = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout>;
  const debouncedHideLastTrade = () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      show_last_trade = false;
    }, 3000);
  };
</script>

<main class="p-2 max-w-[600px] mx-auto">
  {#if socketIsOpen}
    <div class="flex flex-col gap-6 items-stretch">
      <Scores {plants} {trades} />
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
                >Nouveau trade: {trades.at(-1)?.direction === "Buy"
                  ? "achetés"
                  : "vendus"}
                {trades.at(-1)?.volume}MW @ {0.01 *
                  (trades.at(-1)?.price as number)}€ 🤑</span
              >
            </p>
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <p>Not connected</p>
  {/if}
</main>
