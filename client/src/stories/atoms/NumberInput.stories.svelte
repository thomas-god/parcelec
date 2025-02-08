<script module>
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import { none, some } from "../../lib/Options";
  import NumberInput from "../../components/atoms/NumberInput.svelte";

  let value = $state(0);

  const { Story } = defineMeta({
    title: "Atoms/NumberInput",
    component: NumberInput,
    tags: ["autodocs"],
    parameters: {
      layout: "fullscreen",
    },
    args: {
      bigIncr: none(),
      label: "Label",
    },
  });
</script>

<Story name="Default" args={{ value }} />

<Story name="With buttons" args={{ value, bigIncr: some(10) }} />
