<script lang="ts">
  import { Option } from "effect";
  let {
    value = $bindable(),
    label,
    bigIncr = Option.none(),
  }: {
    value: number;
    label: string;
    bigIncr: Option.Option<number>;
  } = $props();

  let showBigIncr = $derived(Option.isSome(bigIncr));
</script>

<div class="flex flex-col items-center p-2 sm:p-4 max-w-64">
  <label for={`number_input_${label}`}>
    <span class="text-gray-700">{label}</span>
  </label>

  <div class="flex flex-row gap-2 items-center">
    {#if showBigIncr}
      <button
        onclick={() => {
          value -= Option.getOrElse(bigIncr, () => 0);
        }}
        id={`number_input_${label}`}
      >
        -{Option.getOrUndefined(bigIncr)}
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
          value += Option.getOrElse(bigIncr, () => 0);
        }}>+{Option.getOrUndefined(bigIncr)}</button
      >
    {/if}
  </div>
</div>
