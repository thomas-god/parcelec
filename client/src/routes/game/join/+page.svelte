<script lang="ts">
import { goto } from "$app/navigation";
import { PUBLIC_APP_URL } from "$env/static/public";
let name = $state("");

const registerPlayer = async () => {
	let rest = await fetch(`${PUBLIC_APP_URL}/game/join`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		mode: "cors",
		credentials: "include",
		body: JSON.stringify({ name }),
	});
	if (rest.status === 201) {
		goto("/game");
	}
};
</script>

<input bind:value={name} type="text" />
<button onclick={() => registerPlayer()}>join</button>
