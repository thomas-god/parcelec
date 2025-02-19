<script module>
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import OrderBookEntry from "../../components/atoms/OrderBookEntry.svelte";

  const { Story } = defineMeta({
    title: "Atoms/OrderBookEntry",
    component: OrderBookEntry,
    tags: ["autodocs"],
    parameters: {},
    args: {},
  });
</script>

<Story
  name="Buy order (bid)"
  args={{
    entry: {
      price: 50,
      volume: 100,
      direction: "Buy",
      owned: false,
      created_at: Date.now().toString(),
      order_id: "toto",
    },
  }}
/>

<Story
  name="Sell order (offer)"
  args={{
    entry: {
      price: 50,
      volume: 100,
      direction: "Sell",
      owned: false,
      created_at: Date.now().toString(),
      order_id: "toto",
    },
  }}
/>

<Story
  name="Owned order"
  args={{
    entry: {
      price: 50,
      volume: 100,
      direction: "Sell",
      owned: true,
      created_at: Date.now().toString(),
      order_id: "toto",
    },
  }}
/>
