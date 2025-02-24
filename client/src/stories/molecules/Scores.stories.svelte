<script module>
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import LastScore from "../../components/molecules/LastScore.svelte";

  const { Story } = defineMeta({
    title: "Molecules/LastScore",
    component: LastScore,
    tags: ["autodocs"],

    args: {},
  });
</script>

<Story
  name="A l'équilibre"
  args={{
    position: 0,
    pnl: 12000,
  }}
/>

<Story
  name="Déficit"
  args={{
    position: -1200,
    pnl: 3000,
  }}
/>

<Story
  name="Surplus"
  args={{
    position: 500,
    pnl: -5500,
  }}
/>
