<script lang="ts">
  import type { Battery } from "$lib/message";

  let { battery }: { battery: Battery } = $props();
</script>

{JSON.stringify(battery)}
