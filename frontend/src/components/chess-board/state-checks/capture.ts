import { ExpandedGameStateAtPointInTime } from "@src/types/board";
import { playCaptureSound } from "../../../sounds";
import { getCapturedPiecesFromBase64 } from "@src/utils";

const didStateChange = (
  one: ExpandedGameStateAtPointInTime,
  two: ExpandedGameStateAtPointInTime,
) => {
  const beforeCapturedPieces = getCapturedPiecesFromBase64(one.capturedPieces);
  const afterCapturedPieces = getCapturedPiecesFromBase64(two.capturedPieces);

  return (
    beforeCapturedPieces.white.length !== afterCapturedPieces.white.length
    || beforeCapturedPieces.black.length !== afterCapturedPieces.black.length
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
