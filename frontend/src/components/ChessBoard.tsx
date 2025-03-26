import { Color, Piece } from "../types/piece";

import "../css/ChessBoard.css";
import { GameRecord } from "../types/game";

interface ChessBoardProps {
  gameRecord: GameRecord;
}

const getPieceLetter = (piece: Piece | null) => {
  if (!piece) return ".";

  const letterMap = {
    king: "K",
    queen: "Q",
    rook: "R",
    bishop: "B",
    knight: "N",
    pawn: "P",
  };

  const letter = letterMap[piece.pieceType] || "?";

  return piece.color === Color.White ? letter : letter.toLowerCase();
};

export const ChessBoard: React.FC<ChessBoardProps> = ({ gameRecord }) => {
  const gameState = gameRecord.game_state;

  return (
    <div>
      <div>Current turn: {gameState.currentTurn}</div>
      <div className="board">
        {gameState.board.squares.map((row, rowIndex) => (
          <div key={rowIndex} className="board-row">
            {row.map((square, colIndex) => (
              <span key={colIndex} className="square">
                {getPieceLetter(square)}
              </span>
            ))}
          </div>
        ))}
      </div>
    </div>
  );
};
