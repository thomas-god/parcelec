<script module>
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import RenewablePlant from "../../components/molecules/RenewablePlant.svelte";

  const { Story } = defineMeta({
    title: "Molecules/RenewablePlant",
    component: RenewablePlant,
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
      type: "RenewablePlant",
      max_power: 700,
      output: {
        setpoint: 250,
        cost: 0,
      },
    },
  }}
/>
