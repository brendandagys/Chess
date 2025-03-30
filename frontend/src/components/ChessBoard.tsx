import { images } from "../images";
import { Color, PieceType } from "../types/piece";
import { Board } from "../types/board";
import { rotateMatrix180Degrees } from "../utils";
import { useDrag } from "../hooks/useDrag";
import { GameRequest } from "../types/api";

import "../css/ChessBoard.css";

interface ChessBoardProps {
  board: Board;
  playerColor: Color;
  gameId: string;
  sendWebSocketMessage: (action: GameRequest) => void;
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
  gameId,
  sendWebSocketMessage,
}) => {
  const shouldRotate = playerColor === Color.Black;
  const board = shouldRotate
    ? rotateMatrix180Degrees(_board.squares)
    : _board.squares;

  const [draggingPiece, handlePointerDown] = useDrag(
    gameId,
    sendWebSocketMessage
  );

  return (
    <div className={`board rank-count-${board.length % 2 ? "odd" : "even"}`}>
      {board.map((row, rowIndex) => {
        const rank =
          1 + (shouldRotate ? rowIndex : board.length - rowIndex - 1);

        return (
          <div key={rowIndex} className="board-row">
            {row.map((piece, colIndex) => {
              const file =
                1 + (shouldRotate ? board.length - colIndex - 1 : colIndex);

              return (
                <div key={colIndex} className="square">
                  {piece ? (
                    <img
                      className="piece"
                      src={imageMap[piece.pieceType][piece.color]}
                      alt={`${piece.color} ${piece.pieceType}`}
                      data-rank={rank}
                      data-file={file}
                      onPointerDown={(e) => {
                        handlePointerDown(e, piece);
                      }}
                    />
                  ) : (
                    <img
                      className="hidden-piece"
                      src={imageMap[PieceType.Pawn][Color.White]}
                      data-rank={rank}
                      data-file={file}
                    />
                  )}
                </div>
              );
            })}
          </div>
        );
      })}

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
