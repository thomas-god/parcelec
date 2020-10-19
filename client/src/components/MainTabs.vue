<template>
  <div class="tabs__container" ref="tabs_container">
    <div
      v-for="cat in categories"
      :key="cat.name"
      :class="cat.name === value ? 'tabs__category_active' : 'tabs__category'"
      @click="update_category(cat.name)"
    >
      <span
        class="tabs__category_logo"
        :style="
          cat.name === value
            ? style_tabs_category_logo_active
            : style_tabs_category_logo
        "
        >{{ cat.logo }}</span
      >
      <span class="tabs__category_name" v-if="display_tab_name">{{
        cat.name
      }}</span>
      <span class="tabs__category_notif" v-if="cat.name === 'Marché' && n_pending_otcs > 0">
        {{ n_pending_otcs }}
      </span>
    </div>
  </div>
</template>

<script lang='ts'>
import { Component, Vue, Prop } from "vue-property-decorator";
import { namespace } from "vuex-class";

const session_module = namespace("session");
const otc_module = namespace("otcs")

@Component
export default class MainTabs extends Vue {
  @Prop({ default: "Home" }) value!: string;
  @session_module.Getter session_multi_game!: boolean;
  @session_module.State results_available!: boolean;
  @otc_module.Getter n_pending_otcs!: number;

  update_category(new_cat: string): void {
    (document.getElementById('main') as HTMLDivElement).scrollIntoView();
    this.$emit("input", new_cat);
  }

  /**
   * Dynamic tabs depending on context
   */
  get categories() {
    const categories = [
      { name: "Home", logo: "🏠" },
      { name: "Centrales", logo: "⚡" },
      { name: "Marché", logo: "⚖️" }
    ];
    if (this.session_multi_game) categories.push({ name: "Chat", logo: "💬" });
    if (this.results_available)
      categories.push({ name: "Résultats", logo: "🏆" });
    return categories;
  }

  /**
   * Monitor component's width to update visibility ratio.
   */
  content_width = 0;
  public handleResize() {
    if (this.$refs.tabs_container)
      this.content_width = (this.$refs
        .tabs_container as HTMLDivElement).clientWidth;
  }
  public mounted() {
    window.addEventListener("resize", this.handleResize);
    if (this.$refs.tabs_container)
      this.content_width = (this.$refs
        .tabs_container as HTMLDivElement).clientWidth;
  }
  public beforeDestroyed() {
    window.removeEventListener("resize", this.handleResize);
  }
  get display_tab_name(): boolean {
    return this.content_width > this.categories.length * (45 + 80);
  }

  /**
   * Dynamic styles
   */
  get style_tabs_category_logo(): string {
    let style = "";
    if (this.content_width < this.categories.length * (45 + 80)) {
      style += "padding: 0.2rem 0.7rem !important;";
    }
    return style;
  }
  get style_tabs_category_logo_active(): string {
    let style = this.style_tabs_category_logo;
    if (this.content_width < this.categories.length * (45 + 80)) {
      style += "background-color: rgba(128, 128, 128, 0.5);";
      style += "border-radius: 0.7rem;";
    }
    return style;
  }
}
</script>

<style scoped>
.tabs__container {
  display: flex;
  flex-direction: row;
  justify-content: space-around;
  align-items: center;

  position: -webkit-sticky;
  position: sticky;
  top: -8px;
  z-index: 12;

  padding: 1rem 0 0.5rem 0;
  border-bottom: 2px solid rgba(0, 0, 0, 0.712);
  background-color: white;
}
.tabs__category,
.tabs__category_active {
  position: relative;
}

.tabs__category:hover .tabs__category_name,
.tabs__category_active .tabs__category_name {
  font-style: italic;
}
.tabs__category_active .tabs__category_name {
  font-weight: bold;
}

.tabs__category_name {
  display: inline-block;
  box-sizing: border-box;
}

.tabs__category_logo {
  display: inline-block;
  width: 45px;
  box-sizing: border-box;
}

.tabs__category_logo,
.tabs__category_name {
  padding: 0 3px;
}

.tabs__category_notif {
  position: absolute;
  font-size: 10px;
  top: -5px;
  right: -12px;
  padding: 2px;
  border-radius: 4px;
  background: red;
  color: white;
  width: 2ch;
  box-sizing: border-box;
}
</style>