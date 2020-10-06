<script lang="ts">
import { ChartData, ChartOptions } from "chart.js";
import { Line } from "vue-chartjs";
import { Component, Vue, Watch } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Bid } from "../store/bids";

const bids_module = namespace("bids");

const options: ChartOptions = {
  maintainAspectRatio: false,
  tooltips: {
    intersect: false
  },
  scales: {
    xAxes: [
      {
        type: "linear",
        ticks: {
          suggestedMin: 0
        },
        scaleLabel: {
          display: true,
          labelString: 'Volume (MWh)'
        } 
      }
    ],
    yAxes: [
      {
        ticks: {
          suggestedMin: 0
        },
        scaleLabel: {
          display: true,
          labelString: 'Prix (€/MWh)'
        } 
      }
    ]
  }
};

@Component({
  extends: Line // this is important to add the functionality to your component
})
export default class CommitChart extends Vue {
  public renderChart!: (chartData: ChartData, options?: ChartOptions) => void;
  @bids_module.Getter all_market_bids!: Bid[];

  get bids_sell(): { x: number; y: number }[] {
    const tmp = this.all_market_bids
      .filter(bid => bid.type === "sell")
      .sort((a, b) => a.price_eur_per_mwh - b.price_eur_per_mwh)
      .map(bid => {
        return { x: bid.volume_mwh, y: bid.price_eur_per_mwh };
      });
    for (let i = 0; i < tmp.length; i++) {
      tmp[i].x += i > 0 ? tmp[i - 1].x : 0;
    }
    if(tmp.length > 0)
      tmp.unshift({ x: 0, y: tmp[0].y });
    return tmp;
  }
  get bids_buy(): { x: number; y: number }[] {
    const tmp = this.all_market_bids
      .filter(bid => bid.type === "buy")
      .sort((a, b) => b.price_eur_per_mwh - a.price_eur_per_mwh)
      .map(bid => {
        return { x: bid.volume_mwh, y: bid.price_eur_per_mwh };
      });
    for (let i = 0; i < tmp.length; i++) {
      tmp[i].x += i > 0 ? tmp[i - 1].x : 0;
    }
    if(tmp.length > 0)
      tmp.unshift({ x: 0, y: tmp[0].y });
    return tmp;
  }

  plot(): void {
    this.renderChart(
      {
        datasets: [
          {
            label: "Ventes",
            backgroundColor: "rgba(0, 0, 0, 0)",
            borderColor: "red",
            data: this.bids_sell,
            steppedLine: "after",
            pointRadius: 0
          },
          {
            label: "Achats",
            backgroundColor: "rgba(0, 0, 0, 0)",
            borderColor: "blue",
            data: this.bids_buy,
            steppedLine: "after",
            pointRadius: 0
          }
        ]
      },
      options
    );
  }

  mounted() {
    // Overwriting base render method with actual data.
    this.plot();
  }
  @Watch("all_market_bids")
  watchBids(): void {
    this.plot();
  }
}
</script>