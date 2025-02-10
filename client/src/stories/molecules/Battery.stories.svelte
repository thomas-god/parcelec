<script module>
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import Battery from "../../components/molecules/Battery.svelte";

  const { Story } = defineMeta({
    title: "Molecules/Battery",
    component: Battery,
    tags: ["autodocs"],
    parameters: {
      layout: "fullscreen",
    },
    args: {},
  });
</script>

<Story
  name="No setpoint"
  args={{
    battery: {
      type: "Battery",
      charge: 350,
      max_charge: 1000,
      current_setpoint: 0,
    },
  }}
/>

<Story
  name="Charge"
  args={{
    battery: {
      type: "Battery",
      charge: 350,
      max_charge: 1000,
      current_setpoint: -250,
    },
  }}
/>

<Story
  name="Discharge"
  args={{
    battery: {
      type: "Battery",
      charge: 350,
      max_charge: 1000,
      current_setpoint: 250,
    },
  }}
/>
