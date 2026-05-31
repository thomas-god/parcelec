<script lang="ts">
  import { goto } from "$app/navigation";
</script>

<div
  class={`
    w-full sticky top-0 z-30 h-18
    text-success-content bg-success rounded-b-none rounded-t-none
    p-4
  `}
>
  <div
    class="w-full mx-auto h-full flex flex-row items-center justify-between @sm:text-base text-sm text-center align-middle"
  >
    <div class="flex flex-row items-center justify-center gap-1.5">
      <button onclick={() => goto("/")}
        ><img
          src="/icons/home.svg"
          alt="homepage icon"
          class="inline h-6 w-6"
        /></button
      >
    </div>
  </div>
</div>
