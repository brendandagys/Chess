import { images } from "../images";
import { Color, Piece, PieceType } from "../types/piece";
import { Board } from "../types/board";
import { rotateBoard180Degrees } from "../utils";

import "../css/ChessBoard.css";
import { useCallback, useEffect, useState } from "react";

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

  const [draggingPiece, setDraggingPiece] = useState<{
    piece: Piece;
    x: number;
    y: number;
  } | null>(null);

  const handleMouseDown = (
    event: React.MouseEvent<HTMLImageElement>,
    piece: Piece
  ) => {
    event.currentTarget.classList.add("dragging");

    setDraggingPiece({
      piece,
      x: event.clientX,
      y: event.clientY,
    });

    event.preventDefault();
  };

  const handleMouseMove = useCallback(
    (event: MouseEvent) => {
      if (draggingPiece) {
        setDraggingPiece((prev) =>
          prev ? { ...prev, x: event.clientX, y: event.clientY } : null
        );
      }
    },
    [draggingPiece]
  );

  const handleMouseUp = useCallback(() => {
    if (draggingPiece) {
      const elementUnderMouse = document.elementFromPoint(
        draggingPiece.x,
        draggingPiece.y
      );

      if (elementUnderMouse && elementUnderMouse instanceof HTMLElement) {
        const classes = elementUnderMouse.classList;
        let pieceElement: HTMLElement | null = null;

        if (classes.contains("square")) {
          pieceElement =
            elementUnderMouse.querySelector(".piece") ??
            elementUnderMouse.querySelector(".hidden-piece");
        } else if (
          classes.contains("piece") ||
          classes.contains("hidden-piece")
        ) {
          pieceElement = elementUnderMouse;
        }

        if (pieceElement) {
          const rank = pieceElement.dataset.rank;
          const file = pieceElement.dataset.file;

          console.log(`Piece placed at rank: ${rank}, file: ${file}`);
        }
      }
    }

    document.querySelectorAll(".piece.dragging").forEach((el) => {
      el.classList.remove("dragging");
    });

    setDraggingPiece(null);
  }, [draggingPiece]);

  useEffect(() => {
    if (draggingPiece) {
      window.addEventListener("mousemove", handleMouseMove);
      window.addEventListener("mouseup", handleMouseUp);
    } else {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    }

    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [draggingPiece, handleMouseMove, handleMouseUp]);

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
              ) / 2 || 0),
            top:
              draggingPiece.y -
              (parseFloat(
                getComputedStyle(document.documentElement).getPropertyValue(
                  "--piece-diameter"
                )
              ) / 2 || 0),
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
