import type { StackSnapshot } from "./message";

type plantType = (StackSnapshot extends Map<any, infer I> ? I : never)["type"];

export const PLANT_ICONS: Record<plantType, string> = {
  GasPlant: "🔥",
  RenewablePlant: "☀️",
  Consumers: "🏙️",
  Battery: "🔋",
  Nuclear: "☢️",
};

export const PLANT_NAMES: Record<plantType, string> = {
  GasPlant: "Centrale gaz",
  RenewablePlant: "Solaire",
  Consumers: "Clients",
  Battery: "Batterie",
  Nuclear: "Centrale nucléaire",
};
