import { ExpandedGameStateAtPointInTime } from "@src/types/board";
import { playCaptureSound } from "../../../sounds";

const didStateChange = (
  one: ExpandedGameStateAtPointInTime,
  two: ExpandedGameStateAtPointInTime,
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
