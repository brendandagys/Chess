import { ExpandedGameStateAtPointInTime } from "@src/types/board";
import { playMoveCheckSound } from "../../../sounds";

const didStateChange = (
  one: ExpandedGameStateAtPointInTime,
  two: ExpandedGameStateAtPointInTime,
) => {
  return !one.inCheck && !!two.inCheck;
};

const action = () => {
  playMoveCheckSound();
};

export const moveCheck = {
  didStateChange,
  action,
  name: "move-check",
};
