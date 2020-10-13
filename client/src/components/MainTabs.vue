<template>
  <div class="tabs_container">
    <div
      v-for="cat in categories"
      :key="cat.name"
      :class="cat.name === value ? 'tabs_category_active' : 'tabs_category'"
      @click="update_category(cat.name)"
    >
      <span class="tabs_category_logo">{{ cat.logo }}</span>
      <span class="tabs_category_name">{{ cat.name }}</span>
    </div>
  </div>
</template>

<script lang='ts'>
import { Component, Vue, Prop } from "vue-property-decorator";

const categories = [
  { name: "Home", logo: "" },
  { name: "Centrales", logo: "⚡" },
  { name: "Marché", logo: "⚖️" },
  { name: "Chat", logo: "💬" }
];

@Component
export default class MainTabs extends Vue {
  @Prop({ default: "Home" }) value!: string;
  categories = categories;

  update_category(new_cat: string): void {
    this.$emit("input", new_cat);
  }

}
</script>

<style scoped>
.tabs_container {
  display: flex;
  flex-direction: row;
  justify-content: space-around;
  align-items: center;

  padding: 1rem 0 0.5rem 0;
  border-bottom: 2px solid rgba(0, 0, 0, 0.712);
}

.tabs_category:hover,
.tabs_category_active {
  font-style: italic;
}
.tabs_category_active {
  font-weight: bold;
}

.tabs_category_logo,
.tabs_category_name {
  padding: 0 3px;
}
</style>