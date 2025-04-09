<script lang="ts">
  let {
    steps,
    steps_index,
    previous_step,
    next_step,
  }: {
    steps: string[];
    steps_index: number;
    previous_step: () => void;
    next_step: () => void;
  } = $props();
</script>

<div class="self-center mb-4 mt-1 min-[400px]:mt-4">
  <div class="join">
    <button
      class="join-item btn"
      onclick={previous_step}
      disabled={steps_index === 0}>«</button
    >
    <button
      class="join-item btn w-46 hover:bg-base-200 hover:border-none transition-none"
      >{steps.at(steps_index)}</button
    >
    <button
      class="join-item btn"
      onclick={next_step}
      disabled={steps_index === steps.length - 1}>»</button
    >
  </div>
</div>
