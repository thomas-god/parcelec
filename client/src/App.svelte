<script lang="ts">
  import svelteLogo from "./assets/svelte.svg";
  import viteLogo from "/vite.svg";
  import Counter from "./lib/Counter.svelte";

  const socket = new WebSocket("ws://127.0.0.1:9002");
  let snapshot = $state({
    bids: [],
    offers: [],
  });
  socket.addEventListener("message", (msg) => {
    let data = JSON.parse(msg.data)["OrderBookSnapshot"];
    console.log(data);
    snapshot = data;
  });
  let orderRequest = {
    price: 100,
    volume: 10,
    direction: "Buy",
    owner: "toto",
  };

  let orderRequestPayload = $derived(
    JSON.stringify({ OrderRequest: orderRequest })
  );
  const sendRequest = () => {
    console.log(`sending order request: ${orderRequestPayload}`);
    socket.send(orderRequestPayload);
  };
</script>

<main>
  <div>
    <a href="https://vite.dev" target="_blank" rel="noreferrer">
      <img src={viteLogo} class="logo" alt="Vite Logo" />
    </a>
    <a href="https://svelte.dev" target="_blank" rel="noreferrer">
      <img src={svelteLogo} class="logo svelte" alt="Svelte Logo" />
    </a>
  </div>
  <h1>Vite + Svelte</h1>

  <div class="card">
    <Counter />
  </div>

  <h1>Order book</h1>
  <ul>
    {#each snapshot.bids as bid}
      <li>{JSON.stringify(bid)}</li>
    {/each}
  </ul>

  <p>
    Check out <a
      href="https://github.com/sveltejs/kit#readme"
      target="_blank"
      rel="noreferrer">SvelteKit</a
    >, the official Svelte app framework powered by Vite!
  </p>

  <button onclick={sendRequest}>Send order</button>

  <p class="read-the-docs">Click on the Vite and Svelte logos to learn more</p>
</main>

<style>
  .logo {
    height: 6em;
    padding: 1.5em;
    will-change: filter;
    transition: filter 300ms;
  }
  .logo:hover {
    filter: drop-shadow(0 0 2em #646cffaa);
  }
  .logo.svelte:hover {
    filter: drop-shadow(0 0 2em #ff3e00aa);
  }
  .read-the-docs {
    color: #888;
  }
</style>
