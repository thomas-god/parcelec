<script module>
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import Consumers from "../../components/molecules/Consumers.svelte";

  const { Story } = defineMeta({
    title: "Molecules/Consumers",
    component: Consumers,
    tags: ["autodocs"],
    parameters: {
      layout: "fullscreen",
    },
    args: {},
  });
</script>

<Story
  name="Default"
  args={{
    plant: {
      type: "Consumers",
      max_power: 1000,
      output: {
        setpoint: 450,
        cost: -450 * 85,
      },
    },
  }}
/>
