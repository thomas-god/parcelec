<script lang="ts">
  import OrderBookComponent from "../../organisms/OrderBook.svelte";
  import type { OrderBook, Trade } from "$lib/message";

  let {
    orderBook,
    send,
    trades,
  }: {
    orderBook: OrderBook;
    send: (msg: string) => void;
    trades: Trade[];
  } = $props();
</script>

<h2 class="font-semibold text-lg mt-4 pl-4">Marché 💱</h2>

<div class="tutorial-container">
  <div>
    <p class="pt-2">
      En plus de vos centrales, le <i>marché </i> 💱 vous permet d'acheter ou de vendre
      de l'énergie avec d'autres joueurs.
    </p>

    <p>
      Si deux offres d'achat et de vente ont le même prix, alors la transaction
      se fait. Sinon, les 2 offres restent sur le marché jusqu'à ce que
      quelqu'un dépose une nouvelle offre ou que la période se termine.
    </p>
    <p class="pb-4">
      Essayer d'acheter l'énergie qu'il vous manque en créant les ordres
      nécessaires sur le marché !
    </p>
  </div>
  <div class="bg-base-100 rounded-lg border-transparent pt-4">
    <OrderBookComponent {orderBook} {trades} {send} />
  </div>
</div>

<style>
  @media (width > 800px) {
    .tutorial-container {
      display: flex;
      flex-direction: row;
      gap: 2rem;
    }

    .tutorial-container > div {
      flex-grow: 1;
      flex: 1;
    }
  }
</style>
