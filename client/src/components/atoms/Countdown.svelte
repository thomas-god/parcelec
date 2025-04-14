<script lang="ts">
  let { end_at }: { end_at: Date } = $props();
  let remaining = $state({ minutes: 0, seconds: 0 });

  $effect(() => {
    const interval = setInterval(updateRemainingTime, 1000);

    return () => {
      clearInterval(interval);
    };
  });

  const updateRemainingTime = () => {
    const now = new Date();
    const endDate = new Date(end_at);

    // Calculate total seconds remaining
    let diff = Math.max(
      0,
      Math.floor((endDate.getTime() - now.getTime()) / 1000),
    );

    // Convert to minutes, seconds
    const minutes = Math.floor(diff / 60);
    diff -= minutes * 60;

    const seconds = diff;

    remaining = { minutes, seconds };
  };
</script>

<span class="countdown font-mono">
  <span
    style="--value:{remaining.minutes};"
    aria-live="polite"
    aria-label="minutes">{remaining.minutes}</span
  >
  m
  <span
    style="--value:{remaining.seconds};"
    aria-live="polite"
    aria-label="seconds">{remaining.seconds}</span
  >
  s
</span>
