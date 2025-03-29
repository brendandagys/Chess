import { images } from "../images";
import { Color, PieceType } from "../types/piece";
import { Board } from "../types/board";
import { rotateBoard180Degrees } from "../utils";

import "../css/ChessBoard.css";
import { useDrag } from "../hooks/useDrag";
interface ChessBoardProps {
  board: Board;
  playerColor: Color;
}

const imageMap = {
  [PieceType.Pawn]: { white: images.wp, black: images.bp },
  [PieceType.Rook]: { white: images.wr, black: images.br },
  [PieceType.Knight]: { white: images.wn, black: images.bn },
  [PieceType.Bishop]: { white: images.wb, black: images.bb },
  [PieceType.Queen]: { white: images.wq, black: images.bq },
  [PieceType.King]: { white: images.wk, black: images.bk },
};

export const ChessBoard: React.FC<ChessBoardProps> = ({
  board: _board,
  playerColor,
}) => {
  const board =
    playerColor === Color.White
      ? rotateBoard180Degrees(_board.squares)
      : _board.squares;

  const [draggingPiece, handleMouseDown] = useDrag();

  return (
    <div className={`board rank-count-${board.length % 2 ? "odd" : "even"}`}>
      {board.map((row, rowIndex) => (
        <div key={rowIndex} className="board-row">
          {row.map((piece, colIndex) => (
            <div key={colIndex} className="square">
              {piece ? (
                <img
                  className="piece"
                  src={imageMap[piece.pieceType][piece.color]}
                  alt={`${piece.color} ${piece.pieceType}`}
                  data-rank={board.length - rowIndex - 1}
                  data-file={colIndex}
                  onMouseDown={(e) => {
                    handleMouseDown(e, piece);
                  }}
                />
              ) : (
                <img
                  className="hidden-piece"
                  src={imageMap[PieceType.Pawn][Color.White]}
                  data-rank={board.length - rowIndex - 1}
                  data-file={colIndex}
                />
              )}
            </div>
          ))}
        </div>
      ))}

      {/* Floating piece */}
      {draggingPiece && (
        <img
          className="dragging-piece"
          src={
            imageMap[draggingPiece.piece.pieceType][draggingPiece.piece.color]
          }
          style={{
            position: "absolute",
            left:
              draggingPiece.x -
              (parseFloat(
                getComputedStyle(document.documentElement).getPropertyValue(
                  "--piece-diameter"
                )
              ) / 2 || 0) +
              window.scrollX,
            top:
              draggingPiece.y -
              (parseFloat(
                getComputedStyle(document.documentElement).getPropertyValue(
                  "--piece-diameter"
                )
              ) / 2 || 0) +
              window.scrollY,
            pointerEvents: "none",
            width: "var(--piece-diameter)",
            height: "var(--piece-diameter)",
            opacity: 1,
          }}
        />
      )}
    </div>
  );
};
