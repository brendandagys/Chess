/* eslint-disable max-len */
import { playCastleSound } from "../../../sounds";
import { GameStateAtPointInTime } from "../../../types/game";
import { PieceType } from "../../../types/piece";

const didStateChange = (
  one: GameStateAtPointInTime,
  two: GameStateAtPointInTime,
) => {
  const numRows = two.board.squares.length;
  const numCols = two.board.squares[0].length;

  return two.board.squares.reduce((castled, row, rowIndex) => (
    castled || row.reduce((rowCastled, square, colIndex) => {
      if (
        [0, numRows - 1].includes(rowIndex)
        && square?.pieceType === PieceType.King
        && (
          (
            colIndex + 2 < numCols
            && one.board.squares[rowIndex][colIndex + 2]?.pieceType === PieceType.King
          )
          ||
          (
            colIndex - 2 >= 0
            && one.board.squares[rowIndex][colIndex - 2]?.pieceType === PieceType.King
          )
        )
      ) {
        return true;
      }
      return rowCastled;
    }, false)
  ), false);
};

const action = () => {
  playCastleSound();
};

export const castle = {
  didStateChange,
  action,
  name: "castle",
};
