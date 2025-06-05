import { playMoveOpponentSound } from "../../../sounds";
import { GameStateAtPointInTime } from "../../../types/game";
import { Color } from "../../../types/piece";

const didStateChange = (
  one: GameStateAtPointInTime,
  two: GameStateAtPointInTime,
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

const name = "move-opponent";

export const moveOpponent = {
  didStateChange,
  action,
  name,
};
