import { useCallback, useEffect, useState } from "react";

import { Piece } from "@src/types/piece";
import { GameRequest } from "@src/types/api";
import { PlayerActionName } from "@src/types/game";
import { API_ROUTE } from "@src/constants";
import { Position } from "@src/types/board";

import "@src/css/ChessBoard.css";

export const useDrag = (
  gameId: string,
  onPointerUp: (action: GameRequest) => void,
  disabled = false
) => {
  const [draggingPiece, setDraggingPiece] = useState<{
    piece: Piece;
    x: number;
    y: number;
  } | null>(null);

  const [from, setFrom] = useState<Position | null>(null);

  const handleDragStart = (
    event:
      | React.DragEvent<HTMLImageElement>
      | React.TouchEvent<HTMLImageElement>,
    piece: Piece
  ) => {
    if (disabled) {
      return;
    }

    // Prevent native drag behavior (which wasn't working in Firefox)
    event.preventDefault();

    document.querySelectorAll(".square--selected").forEach((el) => {
      el.classList.remove("square--selected");
    });

    const elem = event.currentTarget;

    elem.classList.add("dragging");

    if (elem.dataset.rank && elem.dataset.file) {
      setFrom({
        rank: parseInt(elem.dataset.rank),
        file: parseInt(elem.dataset.file),
      });

      let clientX: number, clientY: number;
      if ("touches" in event && event.touches.length > 0) {
        clientX = event.touches[0].clientX;
        clientY = event.touches[0].clientY;
      } else if ("clientX" in event) {
        clientX = event.clientX;
        clientY = event.clientY;
      } else {
        clientX = 0;
        clientY = 0;
      }

      setDraggingPiece({
        piece,
        x: clientX,
        y: clientY,
      });
    }
  };

  const handlePointerMove = useCallback(
    (event: PointerEvent) => {
      if (draggingPiece && !disabled) {
        setDraggingPiece((prev) =>
          prev
            ? {
                ...prev,
                x: event.clientX,
                y: event.clientY,
              }
            : null
        );
      }
    },
    [draggingPiece, disabled]
  );

  const getPieceFromElement = (element: HTMLElement): HTMLElement | null => {
    const classes = element.classList;

    if (classes.contains("square")) {
      return (
        element.querySelector(".piece") ??
        element.querySelector(".hidden-piece")
      );
    } else if (classes.contains("piece") || classes.contains("hidden-piece")) {
      return element;
    }

    return null;
  };

  const handlePointerUp = useCallback(() => {
    if (draggingPiece && !disabled) {
      const elementUnderPointer = document.elementFromPoint(
        draggingPiece.x,
        draggingPiece.y
      );

      let movedPiece = false;

      if (elementUnderPointer && elementUnderPointer instanceof HTMLElement) {
        const piece = getPieceFromElement(elementUnderPointer);

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

            movedPiece = true;

            console.info(`Piece placed at rank: ${toRank}, file: ${toFile}`);
          }
        }
      }

      if (!movedPiece) {
        document.querySelectorAll(".dragging").forEach((el) => {
          el.classList.remove("dragging");
        });
      }

      setDraggingPiece(null);
      setFrom(null);
    }
  }, [draggingPiece, from, gameId, onPointerUp, disabled]);

  useEffect(() => {
    if (draggingPiece && !disabled) {
      document.body.classList.add("dragging-piece");
      window.addEventListener("pointermove", handlePointerMove);
      window.addEventListener("pointerup", handlePointerUp);
    } else {
      document.body.classList.remove("dragging-piece");
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
    }

    return () => {
      document.body.classList.remove("dragging-piece");
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
    };
  }, [draggingPiece, handlePointerMove, handlePointerUp, disabled]);

  return [draggingPiece, handleDragStart] as const;
};
