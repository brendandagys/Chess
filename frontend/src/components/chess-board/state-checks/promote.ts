import { ExpandedGameStateAtPointInTime } from "@src/types/board";
import { playPromoteSound } from "../../../sounds";
import { PieceType } from "../../../types/piece";

const didStateChange = (
  one: ExpandedGameStateAtPointInTime,
  two: ExpandedGameStateAtPointInTime,
) => {
  const getQueenCount = (gameState: ExpandedGameStateAtPointInTime) => (
    gameState.board.squares.reduce((acc, row) => {
      return (
        acc
        + row.filter((square) => square?.pieceType === PieceType.Queen).length
      );
    }, 0)
  );

  return (
    getQueenCount(two) > getQueenCount(one)
  );
};

const action = () => {
  playPromoteSound();
};

export const promote = {
  didStateChange,
  action,
  name: "promote",
};
