<script lang="ts">
  let {
    title,
    min_value,
    value = $bindable(),
    error_message,
  }: {
    title: string;
    min_value: string;
    value: string;
    error_message: string;
  } = $props();
</script>

<label class="fieldset-label flex-col">
  <div class="flex justify-between items-center w-full">
    <div class="self-start text-sm">{title}</div>
  </div>
  <input
    type="number"
    min={min_value}
    class="input validator text-base"
    bind:value
    required
  />
  <p class="validator-hint self-start mt-0">
    {error_message}
  </p>
</label>
