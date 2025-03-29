import "../css/ChessBoard.css";
import { useCallback, useEffect, useState } from "react";
import { Piece } from "../types/piece";

export const useDrag = () => {
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

  return [draggingPiece, handleMouseDown] as const;
};
