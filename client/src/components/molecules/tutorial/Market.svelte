<script lang="ts">
  import OrderBookComponent from "../../organisms/OrderBook.svelte";
  import type { OrderBook, Trade } from "$lib/message";
  import { marketPosition } from "$lib/position";
  import { marketPnl } from "$lib/pnl";

  let {
    orderBook,
    send,
    trades,
  }: {
    orderBook: OrderBook;
    send: (msg: string) => void;
    trades: Trade[];
  } = $props();

  const base_position = 250;
  let position = $derived(marketPosition(trades) + base_position);
  let pnl = $derived(marketPnl(trades));
</script>

<h2 class="font-semibold text-lg mt-4 max-[500px]:pl-4">Marché 💱</h2>

<p class="px-4 pt-2">
  En plus de vos centrales, le <i>marché </i> 💱 vous permet d'acheter ou de vendre
  de l'énergie avec d'autres joueurs.
</p>

<p class="px-4">
  Si deux offres d'achat et de vente ont le même prix, alors la transaction se
  fait. Sinon, les 2 offres restent sur le marché jusqu'à ce que quelqu'un
  dépose une nouvelle offre ou que la période se termine.
</p>
<p class="px-4 pb-4">
  Essayer de vendre votre surplus sur le marché en créer les ordres nécessaires
  !
</p>

<div
  class="bg-base-100 rounded-lg border-transparent pt-4 p-2 mx-auto max-w-96"
>
  <OrderBookComponent {orderBook} {trades} {send} />
</div>
