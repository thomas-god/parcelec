<script lang="ts">
import {
  Chart,
  ChartColor,
  ChartData,
  ChartDataSets,
  ChartOptions,
  ChartPoint,
  Point,
  Scriptable
} from "chart.js";
import { Line, Bar } from "vue-chartjs";
import { Component, Vue, Watch, Prop } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Bid } from "../store/bids";

const bids_module = namespace("bids");

const options: ChartOptions = {
  maintainAspectRatio: false,
  responsive: true,
  legend: {
    position: "bottom"
  },
  tooltips: {
    intersect: false,
    mode: "index",
    axis: "x",
    position: "average",
    filter: (item: Chart.ChartTooltipItem, data: ChartData): boolean => {
      return item.yLabel! !== 0 && item.index! > 0;
    },
    callbacks: {
      title: (item: Chart.ChartTooltipItem[], data: ChartData): string => {
        return "";
      }
    }
  },
  scales: {
    xAxes: [
      {
        scaleLabel: { labelString: "Phase", display: true },
        ticks: {
          suggestedMin: 1
        }
      }
    ],
    yAxes: [
      {
        type: "linear",
        display: true,
        scaleLabel: { labelString: "Energie (MWh)", display: true },
        ticks: {
          suggestedMin: 0,
          maxTicksLimit: 8,
          autoSkip: true
        }
      }
    ]
  }
};

@Component({
  extends: Line
})
export default class MainDataStocksGraph extends Vue {
  @Prop() n_phases!: number;
  @Prop() stocks_by_type!: {
    type: string;
    name: string;
    color: string;
    values: number[];
  }[];
  @Prop() current_phase!: number;
  public renderChart!: (chartData: ChartData, options?: ChartOptions) => void;
  options = options;

  get max_value(): number {
    return (
      Math.ceil(
        this.stocks_by_type
          .map(type =>
            type.values.reduce(
              (prev, cur) => Math.max(prev, cur),
              Number.NEGATIVE_INFINITY
            )
          )
          .reduce(
            (prev, cur) => Math.max(prev, cur),
            Number.NEGATIVE_INFINITY
          ) / 500
      ) * 500
    );
  }
  get min_value(): number {
    return (
      Math.floor(
        this.stocks_by_type
          .map(type =>
            type.values.reduce(
              (prev, cur) => Math.min(prev, cur),
              Number.POSITIVE_INFINITY
            )
          )
          .reduce(
            (prev, cur) => Math.min(prev, cur),
            Number.POSITIVE_INFINITY
          ) / 500
      ) * 500
    );
  }
  get stocks_fmt(): ChartDataSets[] {
    return this.stocks_by_type.map(type => {
      return {
        label: type.name,
        backgroundColor: "rgba(0, 0, 0, 0)",
        borderColor: type.color,
        pointRadius: 0,
        borderJoinStyle: "round",
        steppedLine: false,
        lineTension: 0,
        data: type.values,
        order: 1
      };
    });
  }
  get labels(): string[] {
    const labels = []
    for(let i = 0; i <= this.n_phases; i ++) {
      labels.push(String(i + 1))
    }
    return labels
  }

  plot(): void {
    this.options!.scales!.xAxes![0].ticks!.min = 1;
    this.options!.scales!.xAxes![0].ticks!.max = this.n_phases;
    this.options!.scales!.xAxes![0].ticks!.maxTicksLimit =
      this.n_phases + 1;
    this.options!.scales!.yAxes![0].ticks!.suggestedMin! = this.min_value;
    this.options!.scales!.yAxes![0].ticks!.suggestedMax! = this.max_value;
    this.renderChart(
      {
        datasets: this.stocks_fmt,
        labels: this.labels
      },
      this.options
    );
  }

  mounted() {
    this.plot();
  }
  @Watch("stocks_fmt")
  watchStocks(): void {
    this.plot();
  }
}
</script>