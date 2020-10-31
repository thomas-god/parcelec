<template>
  <div class="tabs__container" ref="tabs_container">
    <div
      v-for="cat in categories"
      :key="cat.name"
      :class="cat.name === value ? 'tabs__category_active' : 'tabs__category'"
      @click="update_category(cat)"
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
      <span class="tabs__category_name" v-if="display_tab_name">
        {{ cat.name }}
      </span>
      <span class="tabs__category_notif" v-if="cat.notif">
        {{
          cat.name === "Marché" ? (n_pending_otcs > 0 ? n_pending_otcs : 1) : 1
        }}
      </span>
    </div>
  </div>
</template>

<script lang='ts'>
import { Component, Vue, Prop } from "vue-property-decorator";
import { namespace, Getter, Mutation } from "vuex-class";
import { Notifications } from "../store";

const session_module = namespace("session");
const otc_module = namespace("otcs");
const portfolio_module = namespace("portfolio");

interface Category {
  name: string;
  logo: string;
  notif?: boolean;
  clear_notif?: () => void;
}

@Component
export default class MainTabs extends Vue {
  @Prop({ default: "Home" }) value!: string;
  @Prop() tabs!: string[];
  @session_module.Getter session_multi_game!: boolean;
  @session_module.State results_available!: boolean;
  @otc_module.Getter n_pending_otcs!: number;
  @Getter notification_chat!: boolean;
  @Getter notification_market!: boolean;
  @Mutation SET_MARKET_NOTIFICATION!: (flag: boolean) => void;
  @Mutation SET_CHAT_NOTIFICATION!: (flag: boolean) => void;
  @portfolio_module.Getter delta_planning!: number;

  update_category(cat: Category): void {
    (document.getElementById("main") as HTMLDivElement).scrollIntoView();
    this.$emit("input", cat.name);
    if (cat.clear_notif) cat.clear_notif();
  }

  /**
   * Dynamic tabs depending on context
   */
  base_categories: Category[] = [
    { name: "Home", logo: "🏠" },
    { name: "Centrales", logo: "⚡", notif: this.delta_planning !== 0 },
    {
      name: "Marché",
      logo: "⚖️",
      notif: this.notification_market,
      clear_notif: () => this.SET_MARKET_NOTIFICATION(false)
    },
    {
      name: "Chat",
      logo: "💬",
      notif: this.notification_chat,
      clear_notif: () => this.SET_CHAT_NOTIFICATION(false)
    },
    { name: "Prévisions", logo: "📊" },
    { name: "Résultats", logo: "🏆" }
  ];
  get categories(): Category[] {
    return this.tabs.map(tab =>
      this.base_categories.find(cat => cat.name === tab) as Category
    );
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
  right: -15px;
  padding: 2px;
  border-radius: 4px;
  background: red;
  color: white;
  width: 2ch;
  box-sizing: border-box;
}
</style>