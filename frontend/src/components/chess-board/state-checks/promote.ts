import { playPromoteSound } from "../../../sounds";
import { GameStateAtPointInTime } from "../../../types/game";
import { PieceType } from "../../../types/piece";

const didStateChange = (
  one: GameStateAtPointInTime,
  two: GameStateAtPointInTime,
) => {
  const getQueenCount = (gameState: GameStateAtPointInTime) => (
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
