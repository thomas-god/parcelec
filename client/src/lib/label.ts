import type { StackSnapshot } from "./message";

export type PlantType = (StackSnapshot extends Map<any, infer I> | null
  ? I
  : never)["type"];

export const PLANT_ICONS: Record<PlantType, string> = {
  GasPlant: "/icons/gas.svg",
  RenewablePlant: "/icons/renewable.svg",
  Consumers: "/icons/consumers.svg",
  Battery: "/icons/storage.svg",
  Nuclear: "/icons/nuclear.svg",
};

export const PLANT_NAMES: Record<PlantType, string> = {
  GasPlant: "Centrale gaz",
  RenewablePlant: "Renouvelables",
  Consumers: "Clients",
  Battery: "Batterie",
  Nuclear: "Centrale nucléaire",
};
