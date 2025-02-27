<script module>
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import CurrentScore from "../../components/molecules/CurrentScore.svelte";

  const { Story } = defineMeta({
    title: "Molecules/CurrentScore",
    component: CurrentScore,
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
