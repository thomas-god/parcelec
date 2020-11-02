<script lang="ts">
import { ChartData, ChartOptions, ChartPoint } from "chart.js";
import { Line } from "vue-chartjs";
import { Component, Vue, Watch, Prop } from "vue-property-decorator";
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
          const val = data!.datasets![tooltipItem!.datasetIndex!].data![
            tooltipItem!.index!
          ] as ChartPoint;
          return `${val.y} MWh`;
        } else return "";
      }
    }
  },
  scales: {
    xAxes: [
      {
        type: "linear",
        ticks: {
          suggestedMin: 1,
          stepSize: 1
        },
        scaleLabel: {
          display: true,
          labelString: "Phase"
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
          labelString: ""
        }
      }
    ]
  }
};
@Component({
  extends: Line
})
export default class MainDataForecastGraph extends Vue {
  @Prop() data!: number[];
  @Prop() line_title!: string;
  @Prop() y_title!: string;
  public renderChart!: (chartData: ChartData, options?: ChartOptions) => void;
  options = options;
  get max_value(): number {
    return (
      Math.ceil(this.data.reduce((max, val) => Math.max(max, val), 0) / 500) *
      500
    );
  }
  get data_fmt(): { x: number; y: number }[] {
    return [{ x: 1, y: this.data[0] }].concat(
      this.data.map((val, id) => {
        return { x: id + 2, y: val };
      })
    );
  }
  plot(): void {
    options!.scales!.xAxes![0].ticks!.min = 1;
    options!.scales!.xAxes![0].ticks!.maxTicksLimit = this.data.length + 1;
    options!.scales!.yAxes![0].ticks!.suggestedMax! = this.max_value;
    options!.scales!.yAxes![0].scaleLabel!.labelString! = this.y_title;
    this.renderChart(
      {
        datasets: [
          {
            label: this.line_title,
            backgroundColor: "rgba(0, 0, 0, 0)",
            borderColor: "rgb(0, 132, 255)",
            data: this.data_fmt,
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
  @Watch("data_fmt")
  watchBids(): void {
    this.plot();
  }
}
</script>