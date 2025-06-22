import { useEffect, useMemo, useRef, useState } from "react";
import { imageMap } from "@src/images";
import { rotateMatrix180Degrees } from "@src/utils";
import { useDrag } from "@src/hooks/useDrag";
import { GameRequest } from "@src/types/api";
import { PlayerActionName } from "@src/types/game";
import { ExpandedGameStateAtPointInTime, Position } from "@src/types/board";
import { Color, Piece } from "@src/types/piece";
import { API_ROUTE } from "@src/constants";
import { stateChecks } from "@src/components/chess-board/state-checks";

import "@src/css/ChessBoard.css";

interface ChessBoardProps {
  expandedHistory: ExpandedGameStateAtPointInTime[];
  playerColor: Color;
  gameId: string;
  sendWebSocketMessage: (action: GameRequest) => void;
  historyIndex: number;
}

export const ChessBoard: React.FC<ChessBoardProps> = ({
  expandedHistory,
  playerColor,
  gameId,
  sendWebSocketMessage,
  historyIndex,
}) => {
  const shouldRotate = playerColor === Color.Black;

  const viewedBoardState = expandedHistory[historyIndex].board;
  const viewedBoardStateSquares = viewedBoardState.squares;

  const boardForRendering = shouldRotate
    ? rotateMatrix180Degrees(viewedBoardStateSquares)
    : viewedBoardStateSquares;

  const numRanks = viewedBoardStateSquares.length;
  const numFiles = viewedBoardStateSquares[0].length;

  const [selectedSquare, setSelectedSquare] = useState<Position | null>(null);

  const isViewingLatestBoard = historyIndex === expandedHistory.length - 1;

  const prevHistoryIndex = useRef<number | null>(null);

  useEffect(() => {
    if (
      prevHistoryIndex.current !== null &&
      prevHistoryIndex.current !== historyIndex
    ) {
      const one = expandedHistory[prevHistoryIndex.current];
      const two = expandedHistory[historyIndex];

      for (const { didStateChange, action } of stateChecks) {
        if (didStateChange(one, two, playerColor)) {
          action();
          break;
        }
      }
    }

    prevHistoryIndex.current = historyIndex;
  }, [historyIndex, expandedHistory, playerColor]);

  const [draggingPiece, handleDragStart] = useDrag(
    gameId,
    sendWebSocketMessage,
    !isViewingLatestBoard
  );

  const pieceDiameterClass =
    window.innerWidth < 400
      ? "--piece-diameter-smallest"
      : window.innerWidth < 450
      ? "--piece-diameter-smaller"
      : window.innerWidth < 500
      ? "--piece-diameter-small"
      : "--piece-diameter";

  const onClickSquare = (
    _event: React.MouseEvent<HTMLDivElement>,
    pieceOnSquare: Piece | null,
    position: Position
  ) => {
    if (!isViewingLatestBoard) {
      setSelectedSquare(null);
      return;
    }

    setSelectedSquare((old) => {
      if (!old) {
        return pieceOnSquare?.color === playerColor ? position : null;
      }

      if (old.rank !== position.rank || old.file !== position.file) {
        sendWebSocketMessage({
          route: API_ROUTE,
          data: {
            [PlayerActionName.MovePiece]: {
              gameId,
              playerMove: {
                from: {
                  rank: old.rank,
                  file: old.file,
                },
                to: {
                  rank: position.rank,
                  file: position.file,
                },
              },
            },
          },
        });
      }

      return null;
    });
  };

  const lastMoveSquares: Position[] = useMemo(() => {
    const squares: Position[] = [];

    if (historyIndex > 0) {
      const one = expandedHistory[historyIndex - 1].board.squares;
      const two = viewedBoardStateSquares;

      two.forEach((row, rowIndex) => {
        row.forEach((piece, colIndex) => {
          const oldPiece = one[rowIndex][colIndex];

          if (
            (!piece && oldPiece) ||
            (piece && !oldPiece) ||
            (piece &&
              (piece.color !== oldPiece?.color ||
                piece.pieceType !== oldPiece.pieceType))
          ) {
            squares.push({
              rank: numRanks - rowIndex,
              file: 1 + colIndex,
            });
          }
        });
      });
    }

    return squares;
  }, [historyIndex, expandedHistory, viewedBoardStateSquares, numRanks]);

  const rankNumberToLetterMap: Record<number, string> = {
    1: "A",
    2: "B",
    3: "C",
    4: "D",
    5: "E",
    6: "F",
    7: "G",
    8: "H",
    9: "I",
    10: "J",
    11: "K",
    12: "L",
  };

  interface PositionLabel {
    rankLabel: string;
    fileLabel: string;
  }

  /** Get rank and file labels for a square. */
  const getPositionLabel = (rank: number, file: number): PositionLabel => {
    const isLeftMostFile =
      (playerColor === Color.Black && file === numFiles) ||
      (playerColor === Color.White && file === 1);

    const isBottomRank =
      (playerColor === Color.Black && rank === numRanks) ||
      (playerColor === Color.White && rank === 1);

    return {
      rankLabel: isLeftMostFile ? rankNumberToLetterMap[rank] : "",
      fileLabel: isBottomRank ? `${file}` : "",
    };
  };

  useEffect(() => {
    if (selectedSquare) {
      const pieceOnSelectedSquare =
        viewedBoardStateSquares[numRanks - selectedSquare.rank][
          selectedSquare.file - 1
        ];

      if (pieceOnSelectedSquare?.color !== playerColor) {
        setSelectedSquare(null);
      }
    }
  }, [
    historyIndex,
    numRanks,
    playerColor,
    selectedSquare,
    viewedBoardStateSquares,
  ]);

  return (
    <div className={`board rank-count-${numRanks % 2 ? "odd" : "even"}`}>
      {boardForRendering.map((row, rowIndex) => {
        const rank = 1 + (shouldRotate ? rowIndex : numRanks - rowIndex - 1);

        return (
          <div key={rowIndex} className="board-row">
            {row.map((piece, colIndex) => {
              const file =
                1 + (shouldRotate ? numFiles - colIndex - 1 : colIndex);

              const { rankLabel, fileLabel } = getPositionLabel(rank, file);

              return (
                <div
                  key={colIndex}
                  className={`square${
                    isViewingLatestBoard &&
                    selectedSquare?.rank === rank &&
                    selectedSquare.file === file
                      ? " square--selected"
                      : ""
                  }${
                    lastMoveSquares.some(
                      (move) => move.rank === rank && move.file === file
                    )
                      ? (rank + file) % 2 === 0
                        ? " square--previous-move-dark-square"
                        : " square--previous-move-light-square"
                      : ""
                  }`}
                  onClick={(e) => {
                    onClickSquare(e, piece, { rank, file });
                  }}
                >
                  {
                    <>
                      <span className="position-label rank-label">
                        {rankLabel}
                      </span>

                      <span className="position-label file-label">
                        {fileLabel}
                      </span>
                    </>
                  }
                  {piece ? (
                    <img
                      className="piece"
                      src={imageMap[piece.pieceType][piece.color]}
                      alt={`${piece.color} ${piece.pieceType}`}
                      data-rank={rank}
                      data-file={file}
                      onDragStart={(e) => {
                        if (piece.color === playerColor) {
                          handleDragStart(e, piece);
                        }
                      }}
                      onTouchMove={(e) => {
                        if (piece.color === playerColor) {
                          handleDragStart(e, piece);
                        }
                      }}
                    />
                  ) : (
                    <img
                      className="hidden-piece"
                      width="150" // Same size as the piece PNGs
                      height="150"
                      data-rank={rank}
                      data-file={file}
                      // eslint-disable-next-line max-len
                      src="data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='150' height='150'></svg>"
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
          src={
            imageMap[draggingPiece.piece.pieceType][draggingPiece.piece.color]
          }
          className="floating-piece"
          style={{
            position: "absolute",
            left:
              draggingPiece.x -
              (parseFloat(
                getComputedStyle(document.documentElement).getPropertyValue(
                  pieceDiameterClass
                )
              ) / 2 || 0) +
              window.scrollX,
            top:
              draggingPiece.y -
              (parseFloat(
                getComputedStyle(document.documentElement).getPropertyValue(
                  pieceDiameterClass
                )
              ) / 2 || 0) +
              window.scrollY,
            pointerEvents: "none",
            width: `var(${pieceDiameterClass})`,
            height: `var(${pieceDiameterClass})`,
            opacity: 1,
          }}
        />
      )}
    </div>
  );
};
