<script lang="ts">
  import { goto } from "$app/navigation";
  import { PUBLIC_APP_URL } from "$env/static/public";
  import { parseMessage, type OrderBook, type Trade } from "$lib/message";
  import { match } from "ts-pattern";
  import Intro from "../../components/molecules/tutorial.svelte/Intro.svelte";
  import Market from "../../components/molecules/tutorial.svelte/Market.svelte";
  import PeriodsAndForecasts from "../../components/molecules/tutorial.svelte/PeriodsAndForecasts.svelte";
  import PowerPlants from "../../components/molecules/tutorial.svelte/PowerPlants.svelte";
  import { isSome, none, some, type Option } from "$lib/Options";

  let error = $state(false);
  let orderBook: OrderBook = $state({
    bids: [],
    offers: [],
  });
  let trades: Trade[] = $state([]);
  let trades_to_display: Trade[] = $state([]);
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

  const sendMessage = (msg: string) => {
    if (isSome(game_socket)) {
      game_socket.value.send(msg);
    }
  };

  const steps = [
    "Introduction",
    "Centrales",
    "Marché",
    "Périodes et prévisions",
  ];
  let steps_index = $state(0);
  let current_step = $derived(steps.at(steps_index));
  const next_step = () => {
    if (steps_index < steps.length - 1) {
      steps_index += 1;
    }
  };
  const previous_step = () => {
    if (steps_index > 0) {
      steps_index -= 1;
    }
  };
</script>

{#await startTutorial() then}
  <div class="flex flex-col max-w-[500px] mx-auto text-justify">
    <h1 class="text-center font-semibold text-xl my-3">
      Bienvenue dans Parcelec !
    </h1>

    {#if current_step === "Introduction"}
      <Intro />
    {:else if current_step === "Centrales"}
      <PowerPlants />
    {:else if current_step === "Marché"}
      <Market {orderBook} {trades} send={sendMessage} />
    {:else if current_step === "Périodes et prévisions"}
      <PeriodsAndForecasts />
      <button onclick={() => goto("/game")} class="text-lg mt-3 mb-5"
        >➡️ Commencer une partie</button
      >
    {/if}

    <div class="divider px-3"></div>
    <div class="self-center">
      <div class="join">
        <button
          class="join-item btn"
          onclick={previous_step}
          disabled={steps_index === 0}>«</button
        >
        <button
          class="join-item btn w-46 hover:bg-base-200 hover:border-none transition-none"
          >{steps.at(steps_index)}</button
        >
        <button
          class="join-item btn"
          onclick={next_step}
          disabled={steps_index === steps.length - 1}>»</button
        >
      </div>
    </div>
  </div>

  {#if error}
    <p>Erreur lors de la création du tutoriel</p>
  {/if}
{/await}
