import { ExpandedGameStateAtPointInTime } from "@src/types/board";
import { playMoveOpponentSound } from "../../../sounds";
import { Color } from "../../../types/piece";

const didStateChange = (
  one: ExpandedGameStateAtPointInTime,
  two: ExpandedGameStateAtPointInTime,
  playerColor: Color,
) => {
  return (
    two.currentTurn === playerColor &&
    !one.board.squares.every((row, r) =>
      row.every((oldPiece, c) => {
        const newBoardPiece = two.board.squares[r][c];

        return (
          newBoardPiece?.color === oldPiece?.color &&
          newBoardPiece?.pieceType === oldPiece?.pieceType
        );
      })
    )
  );
};

const action = () => {
  playMoveOpponentSound();
};

export const moveOpponent = {
  didStateChange,
  action,
  name: "move-opponent",
};
