import { playCaptureSound } from "../../../sounds";
import { GameStateAtPointInTime } from "../../../types/game";

const didStateChange = (
  one: GameStateAtPointInTime,
  two: GameStateAtPointInTime,
) => {
  return (
    one.capturedPieces.white.length != two.capturedPieces.white.length
    || one.capturedPieces.black.length != two.capturedPieces.black.length
  );
};

const action = () => {
  playCaptureSound();
};

export const capture = {
  didStateChange,
  action,
  name: "capture",
};
