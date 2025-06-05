import { playMoveCheckSound } from "../../../sounds";
import { GameStateAtPointInTime } from "../../../types/game";

const didStateChange = (
  one: GameStateAtPointInTime,
  two: GameStateAtPointInTime,
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
