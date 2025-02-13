<script lang="ts">
  // based on suggestions from:
  // Inclusive Components by Heydon Pickering https://inclusive-components.design/toggle-button/
  // On Designing and Building Toggle Switches by Sara Soueidan https://www.sarasoueidan.com/blog/toggle-switch-design/
  // and this example by Scott O'hara https://codepen.io/scottohara/pen/zLZwNv

  let {
    on_label,
    off_label,
    checked = $bindable(),
    onInput,
  }: {
    on_label: string;
    off_label: string;
    checked: boolean;
    onInput: () => void;
  } = $props();

  function handleClick(event: any) {
    const target = event.target;

    const state = target.getAttribute("aria-checked");

    checked = state === "true" ? false : true;
    onInput();
  }
</script>

<div class="s s--inner">
  <button role="switch" aria-checked={checked} onclick={handleClick}>
    <span>{on_label}</span>
    <span>{off_label}</span>
  </button>
</div>

<style>
  :root {
    --accent-color: CornflowerBlue;
    --gray: #ccc;
  }
  /* Inner Design Option */
  .s--inner button {
    padding: 0.5em;
    background-color: #fff;
    border: 1px solid var(--gray);
  }
  [role="switch"][aria-checked="true"] :first-child,
  [role="switch"][aria-checked="false"] :last-child {
    display: none;
    color: #fff;
  }

  .s--inner button span {
    user-select: none;
    pointer-events: none;
    padding: 0.25em;
  }

  .s--inner button:focus {
    outline: var(--accent-color) solid 1px;
  }

  /* gravy */

  /* Inner Design Option */
  [role="switch"][aria-checked="true"] :first-child,
  [role="switch"][aria-checked="false"] :last-child {
    border-radius: 0.25em;
    background: var(--accent-color);
    display: inline-block;
  }

  .s--inner button:focus {
    box-shadow: 0 0px 8px var(--accent-color);
    border-radius: 0.1em;
  }
</style>
