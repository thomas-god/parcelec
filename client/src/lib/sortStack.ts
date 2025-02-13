import { match, Pattern } from "ts-pattern";
import type { StackSnapshot } from "./message";

export const sortStack = (stack: StackSnapshot) => {
  return new Map(
    [...stack.entries()].sort((a, b) =>
      match([a, b])
        .with(
          [[Pattern.any, { type: "Consumers" }], Pattern.any],
          ([a, b]) => -1,
        )
        .with(
          [Pattern.any, [Pattern.any, { type: "Consumers" }]],
          ([a, b]) => 1,
        )
        .with([Pattern.any, Pattern.any], ([[a_id, _], [b_id, __]]) => {
          if (a_id < b_id) {
            return -1;
          } else if (a_id > b_id) {
            return 1;
          } else {
            return 0;
          }
        })
        .otherwise(() => -1),
    ),
  );
};
