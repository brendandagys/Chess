import wp from "../images/wp.png";
import bp from "../images/bp.png";
import wr from "../images/wr.png";
import br from "../images/br.png";
import wn from "../images/wn.png";
import bn from "../images/bn.png";
import wb from "../images/wb.png";
import bb from "../images/bb.png";
import wq from "../images/wq.png";
import bq from "../images/bq.png";
import wk from "../images/wk.png";
import bk from "../images/bk.png";

import "../css/ChessBoard.css";
import { Color, PieceType } from "../types/piece";
import { Board } from "../types/board";
import { rotateBoard180Degrees } from "../utils";

interface ChessBoardProps {
  board: Board;
  playerColor: Color;
}

const imageMap = {
  [PieceType.Pawn]: { white: wp, black: bp },
  [PieceType.Rook]: { white: wr, black: br },
  [PieceType.Knight]: { white: wn, black: bn },
  [PieceType.Bishop]: { white: wb, black: bb },
  [PieceType.Queen]: { white: wq, black: bq },
  [PieceType.King]: { white: wk, black: bk },
};

export const ChessBoard: React.FC<ChessBoardProps> = ({
  board,
  playerColor,
}) => {
  return (
    <div
      className={`board rank-count-${
        board.squares.length % 2 ? "odd" : "even"
      }`}
    >
      {(playerColor === Color.White
        ? rotateBoard180Degrees(board.squares)
        : board.squares
      ).map((row, rowIndex) => (
        <div key={rowIndex} className="board-row">
          {row.map((piece, colIndex) => (
            <div key={colIndex} className="square">
              {piece ? (
                <img
                  className="piece"
                  src={imageMap[piece.pieceType][piece.color]}
                  alt={`${piece.color} ${piece.pieceType}`}
                />
              ) : (
                <img
                  className="hidden-piece"
                  src={imageMap[PieceType.Pawn][Color.White]}
                />
              )}
            </div>
          ))}
        </div>
      ))}
    </div>
  );
};
