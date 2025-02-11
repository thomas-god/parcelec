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
  import Position from "../../components/molecules/Position.svelte";

  let orderBook: OrderBook = $state({
    bids: [],
    offers: [],
  });
  let trades: Trade[] = $state([]);
  let plants: StackSnapshot = $state(new Map());

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
</script>

<main class="p-2 max-w-[600px]">
  {#if socketIsOpen}
    <div class="flex flex-col gap-6">
      <Position {plants} {trades} />
      <Stack {plants} send={sendMessage} />
      <OrderBookElement {orderBook} send={sendMessage} />
    </div>
  {:else}
    <p>Not connected</p>
  {/if}
</main>
