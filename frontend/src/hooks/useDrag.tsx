import { useCallback, useEffect, useState } from "react";
import { Piece } from "../types/piece";
import { GameRequest } from "../types/api";
import { PlayerActionName } from "../types/game";
import { API_ROUTE } from "../constants";
import { Position } from "../types/board";

import "../css/ChessBoard.css";

export const useDrag = (
  gameId: string,
  onMouseUp: (action: GameRequest) => void
) => {
  const [draggingPiece, setDraggingPiece] = useState<{
    piece: Piece;
    x: number;
    y: number;
  } | null>(null);

  const [from, setFrom] = useState<Position | null>(null);

  const handleMouseDown = (
    event: React.MouseEvent<HTMLImageElement>,
    piece: Piece
  ) => {
    const elem = event.currentTarget;

    elem.classList.add("dragging");

    if (elem.dataset.rank && elem.dataset.file) {
      setFrom({
        rank: parseInt(elem.dataset.rank),
        file: parseInt(elem.dataset.file),
      });
    }

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
        let piece: HTMLElement | null = null;

        if (classes.contains("square")) {
          piece =
            elementUnderMouse.querySelector(".piece") ??
            elementUnderMouse.querySelector(".hidden-piece");
        } else if (
          classes.contains("piece") ||
          classes.contains("hidden-piece")
        ) {
          piece = elementUnderMouse;
        }

        if (from && piece?.dataset.rank && piece.dataset.file) {
          const toRank = parseInt(piece.dataset.rank);
          const toFile = parseInt(piece.dataset.file);

          onMouseUp({
            route: API_ROUTE,
            data: {
              [PlayerActionName.MovePiece]: {
                gameId,
                playerMove: {
                  from: {
                    rank: from.rank,
                    file: from.file,
                  },
                  to: {
                    rank: toRank,
                    file: toFile,
                  },
                },
              },
            },
          });

          console.log(`Piece placed at rank: ${toRank}, file: ${toFile}`);
        }
      }
    }

    document.querySelectorAll(".piece.dragging").forEach((el) => {
      el.classList.remove("dragging");
    });

    setDraggingPiece(null);
    setFrom(null);
  }, [draggingPiece, from, gameId, onMouseUp]);

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
