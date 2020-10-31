<script lang="ts">
import { ChartData, ChartOptions, ChartPoint } from "chart.js";
import { Line } from "vue-chartjs";
import { Component, Vue, Watch } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Bid } from "../store/bids";

const bids_module = namespace("bids");

const options: ChartOptions = {
  maintainAspectRatio: false,
  responsive: true,
  tooltips: {
    intersect: false,
    mode: "nearest",
    axis: "y",
    position: "nearest",
    filter: (item: Chart.ChartTooltipItem, data: ChartData): boolean => {
      return item.index! > 0;
    },
    callbacks: {
      title: (item: Chart.ChartTooltipItem[], data: ChartData): string => {
        return "";
      },
      label: (tooltipItem: Chart.ChartTooltipItem, data: ChartData): string => {
        if (tooltipItem.index! > 0) {
          const type = data!.datasets![tooltipItem!.datasetIndex!].label!;
          const val = data!.datasets![tooltipItem!.datasetIndex!].data![
            tooltipItem!.index!
          ] as ChartPoint;
          const prev_val = data!.datasets![tooltipItem!.datasetIndex!].data![
            tooltipItem!.index! - 1
          ] as ChartPoint;
          return `${type} : ${(val.x as number) -
            (prev_val.x as number)} MWh à ${val.y} €/MWh`;
        } else return "";
      }
    }
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
          labelString: "Volume (MWh)"
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
          labelString: "Prix (€/MWh)"
        }
      }
    ]
  }
};

@Component({
  extends: Line // this is important to add the functionality to your component
})
export default class BidsGraphAll extends Vue {
  public renderChart!: (chartData: ChartData, options?: ChartOptions) => void;
  @bids_module.Getter all_market_bids!: Bid[];
  options = options;

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
    if (tmp.length > 0) tmp.unshift({ x: 0, y: tmp[0].y });
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
    if (tmp.length > 0) tmp.unshift({ x: 0, y: tmp[0].y });
    return tmp;
  }

  get max_price(): number {
    const max = Math.max(
      this.bids_buy.reduce(
        (max, val) => Math.max(max, val.y),
        Number.NEGATIVE_INFINITY
      ),
      this.bids_sell.reduce(
        (max, val) => Math.max(max, val.y),
        Number.NEGATIVE_INFINITY
      )
    );
    return max !== Number.NEGATIVE_INFINITY ? max : 0;
  }

  plot(): void {
    options!.scales!.yAxes![0].ticks!.suggestedMax! = this.max_price + 10;
    this.renderChart(
      {
        datasets: [
          {
            label: "Ventes",
            backgroundColor: "rgba(0, 0, 0, 0)",
            borderColor: "rgb(0, 132, 255)",
            data: this.bids_sell,
            steppedLine: "after",
            pointRadius: 0
          },
          {
            label: "Achats",
            backgroundColor: "rgba(0, 0, 0, 0)",
            borderColor: "green",
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