import { useCallback, useEffect, useState } from "react";
import { Piece } from "../types/piece";
import { GameRequest } from "../types/api";
import { PlayerActionName } from "../types/game";
import { API_ROUTE } from "../constants";
import { Position } from "../types/board";

import "../css/ChessBoard.css";

export const useDrag = (
  gameId: string,
  onPointerUp: (action: GameRequest) => void
) => {
  const [draggingPiece, setDraggingPiece] = useState<{
    piece: Piece;
    x: number;
    y: number;
  } | null>(null);

  const [from, setFrom] = useState<Position | null>(null);

  const handlePointerDown = (
    event: React.PointerEvent<HTMLImageElement>,
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

  const handlePointerMove = useCallback(
    (event: PointerEvent) => {
      if (draggingPiece) {
        setDraggingPiece((prev) =>
          prev ? { ...prev, x: event.clientX, y: event.clientY } : null
        );
      }
    },
    [draggingPiece]
  );

  const handlePointerUp = useCallback(() => {
    if (draggingPiece) {
      const elementUnderPointer = document.elementFromPoint(
        draggingPiece.x,
        draggingPiece.y
      );

      if (elementUnderPointer && elementUnderPointer instanceof HTMLElement) {
        const classes = elementUnderPointer.classList;
        let piece: HTMLElement | null = null;

        if (classes.contains("square")) {
          piece =
            elementUnderPointer.querySelector(".piece") ??
            elementUnderPointer.querySelector(".hidden-piece");
        } else if (
          classes.contains("piece") ||
          classes.contains("hidden-piece")
        ) {
          piece = elementUnderPointer;
        }

        if (from && piece?.dataset.rank && piece.dataset.file) {
          const toRank = parseInt(piece.dataset.rank);
          const toFile = parseInt(piece.dataset.file);

          // Prohibit same-square moves
          if (toRank !== from.rank || toFile !== from.file) {
            onPointerUp({
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
    }

    document.querySelectorAll(".piece.dragging").forEach((el) => {
      el.classList.remove("dragging");
    });

    setDraggingPiece(null);
    setFrom(null);
  }, [draggingPiece, from, gameId, onPointerUp]);

  useEffect(() => {
    if (draggingPiece) {
      window.addEventListener("pointermove", handlePointerMove);
      window.addEventListener("pointerup", handlePointerUp);
    } else {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
    }

    return () => {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
    };
  }, [draggingPiece, handlePointerMove, handlePointerUp]);

  return [draggingPiece, handlePointerDown] as const;
};
