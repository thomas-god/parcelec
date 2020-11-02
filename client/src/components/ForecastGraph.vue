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
  tooltips: {
    intersect: false,
    mode: "index",
    axis: "x",
    position: "average",
    filter: (item: Chart.ChartTooltipItem, data: ChartData): boolean => {
      return item.index! > 0;
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
        id: "yc",
        display: true,
        scaleLabel: { labelString: "Puissance (MW)", display: true },
        ticks: {
          suggestedMin: 0,
          suggestedMax: 2500
        }
      },
      {
        type: "linear",
        id: "yp",
        display: false,
        stacked: true,
        gridLines: {
          drawOnChartArea: false
        },
        ticks: {
          suggestedMin: 0,
          suggestedMax: 2500
        }
      }
    ]
  }
};

@Component({
  extends: Line
})
export default class ForecastGraph extends Vue {
  @Prop() conso!: number[];
  @Prop() plannings_by_type!: {
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
        Math.max(
          this.conso.reduce((max, val) => Math.max(max, val), 0),
          this.plannings_by_type
            .map(type =>
              type.values.reduce(
                (prev, cur) => Math.max(prev, cur),
                Number.NEGATIVE_INFINITY
              )
            )
            .reduce(
              (prev, cur) => Math.max(prev, cur),
              Number.NEGATIVE_INFINITY
            )
        ) / 500
      ) * 500
    );
  }
  get conso_fmt(): { x: number; y: number }[] {
    return [{ x: 1, y: this.conso[0] }].concat(
      this.conso.map((val, id) => {
        return { x: id + 2, y: val };
      })
    );
  }
  get plannings_fmt(): ChartDataSets[] {
    return this.plannings_by_type.map(type => {
      return {
        label: type.name,
        stack: "Prod",
        borderColor: "rgba(0, 0, 0, 0)",
        backgroundColor: type.color,
        pointRadius: 0,
        steppedLine: "after",
        data: [type.values[0]].concat(type.values),
        order: 1,
        yAxisID: "yp"
      };
    });
  }

  plot(): void {
    this.options!.scales!.xAxes![0].ticks!.min = 1;
    this.options!.scales!.xAxes![0].ticks!.maxTicksLimit =
      this.conso.length + 1;
    this.options!.scales!.yAxes![0].ticks!.suggestedMax! = this.max_value;
    this.options!.scales!.yAxes![0].ticks!.min = 0;
    this.renderChart(
      {
        datasets: [
          ...this.plannings_fmt,
          {
            label: "Consommation",
            stack: "Conso",
            backgroundColor: "rgba(0, 0, 0, 0)",
            borderColor: "rgb(0, 132, 255)",
            data: this.conso_fmt,
            steppedLine: "after",
            pointRadius: 0,
            order: 2,
            yAxisID: "yc"
          }
        ],
        labels: ["1", "2", "3", "4"]
      },
      this.options
    );
  }

  mounted() {
    this.plot();
  }
  @Watch("conso_fmt")
  watchConso(): void {
    this.plot();
  }
  @Watch("plannings_fmt")
  watchPlannings(): void {
    this.plot();
  }
}
</script>