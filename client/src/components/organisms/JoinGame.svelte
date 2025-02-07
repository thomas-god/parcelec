<script lang="ts">
	import {PUBLIC_APP_URL} from '$env/static/public';
let name = $state("");

const registerPlayer = async () => {
	await fetch(`${PUBLIC_APP_URL}/game/join`, {
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
