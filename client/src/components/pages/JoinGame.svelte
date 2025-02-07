<script lang="ts">
let name = $state("");

const registerPlayer = async () => {
	await fetch(`${import.meta.env.VITE_APP_URL}/game/join`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		mode: "cors",
		credentials: "include",
		body: JSON.stringify({ name }),
	});
};
</script>

<input bind:value={name} type="text" />
<button onclick={() => registerPlayer()}>join</button>
