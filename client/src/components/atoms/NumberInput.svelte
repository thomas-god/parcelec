<script lang="ts">
import { isSome, none, unwrap, type Option } from "../../lib/Options";

let {
	value = $bindable(),
	label,
	bigIncr = none(),
}: {
	value: number;
	label: string;
	bigIncr: Option<number>;
} = $props();

let showBigIncr = $derived(isSome(bigIncr));
</script>

<div class="flex flex-col items-center p-2 sm:p-4 max-w-64">
  <label for={`number_input_${label}`}>
    <span class="text-gray-700">{label}</span>
  </label>

  <div class="flex flex-row gap-2 items-center">
    {#if showBigIncr}
      <button
        onclick={() => {
          value -= unwrap(bigIncr);
        }}
        id={`number_input_${label}`}
      >
        -{unwrap(bigIncr)}
      </button>
    {/if}
    <input
      type="number"
      bind:value
      class="block w-full rounded border-gray-300 shadow-sm p-2 text-center"
    />
    {#if showBigIncr}
      <button
        onclick={() => {
          value += unwrap(bigIncr);
        }}>+{unwrap(bigIncr)}</button
      >
    {/if}
  </div>
</div>
