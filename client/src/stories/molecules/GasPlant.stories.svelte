<script module>
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import GasPlant from "../../components/molecules/GasPlant.svelte";

  const { Story } = defineMeta({
    title: "Molecules/GasPlant",
    component: GasPlant,
    tags: ["autodocs"],
    parameters: {
      layout: "fullscreen",
    },
    args: {},
  });
</script>

<Story
  name="Pas de consigne"
  args={{
    plant: {
      type: "GasPlant",
      output: {
        cost: 0,
        setpoint: 0,
      },
      settings: {
        energy_cost: 85,
        max_setpoint: 800,
      },
    },
  }}
/>

<Story
  name="Avec consigne"
  args={{
    plant: {
      type: "GasPlant",
      output: {
        cost: 550 * 85,
        setpoint: 550,
      },
      settings: {
        energy_cost: 85,
        max_setpoint: 800,
      },
    },
  }}
/>
