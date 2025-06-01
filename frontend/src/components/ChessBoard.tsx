import { useState } from "react";
import { imageMap } from "../images";
import { rotateMatrix180Degrees } from "../utils";
import { useDrag } from "../hooks/useDrag";
import { GameRequest } from "../types/api";
import { GameState, PlayerActionName } from "../types/game";
import { Position } from "../types/board";
import { Color, Piece, PieceType } from "../types/piece";
import { API_ROUTE } from "../constants";

import "../css/ChessBoard.css";

interface ChessBoardProps {
  gameState: GameState;
  playerColor: Color;
  gameId: string;
  sendWebSocketMessage: (action: GameRequest) => void;
  boardHistoryIndex: number;
}

export const ChessBoard: React.FC<ChessBoardProps> = ({
  gameState,
  playerColor,
  gameId,
  sendWebSocketMessage,
  boardHistoryIndex,
}) => {
  const shouldRotate = playerColor === Color.Black;

  const viewedBoardState = gameState.boardHistory[boardHistoryIndex];
  const viewedBoardStateSquares = viewedBoardState.squares;

  const boardForRendering = shouldRotate
    ? rotateMatrix180Degrees(viewedBoardStateSquares)
    : viewedBoardStateSquares;

  const numRanks = viewedBoardStateSquares.length;
  const numFiles = viewedBoardStateSquares[0].length;

  const [selectedSquare, setSelectedSquare] = useState<Position | null>(null);

  const isViewingLatestBoard =
    boardHistoryIndex === gameState.boardHistory.length - 1;

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
        return pieceOnSquare ? position : null;
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

  const lastMoveSquares: Position[] = [];

  if (gameState.boardHistory.length >= 2) {
    const one = gameState.boardHistory[gameState.boardHistory.length - 2];
    const two = gameState.boardHistory[gameState.boardHistory.length - 1];

    two.squares.forEach((row, rowIndex) => {
      row.forEach((piece, colIndex) => {
        const oldPiece = one.squares[rowIndex][colIndex];

        if (
          (!piece && oldPiece) ||
          (piece && !oldPiece) ||
          (piece &&
            (piece.color !== oldPiece?.color ||
              piece.pieceType !== oldPiece.pieceType))
        ) {
          lastMoveSquares.push({
            rank: numRanks - rowIndex,
            file: 1 + colIndex,
          });
        }
      });
    });
  }

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

  /**
   * Get position label for a square from its rank, file, and player color.
   * Labels show only for the bottom-left-most square from the player's POV.
   * Also returns whether the current square is the bottom-left-most square.
   */
  const getPositionLabel = (rank: number, file: number): [string, boolean] => {
    const isLeftMostFile =
      (playerColor === Color.Black && file === numFiles) ||
      (playerColor === Color.White && file === 1);

    const isBottomRank =
      (playerColor === Color.Black && rank === numRanks) ||
      (playerColor === Color.White && rank === 1);

    if (isLeftMostFile && isBottomRank) {
      return [`${rankNumberToLetterMap[file]}${rank}`, true];
    }

    if (isLeftMostFile) {
      return [`${rank}`, false];
    }

    if (isBottomRank) {
      return [rankNumberToLetterMap[file], false];
    }

    return ["", false];
  };

  return (
    <div className={`board rank-count-${numRanks % 2 ? "odd" : "even"}`}>
      {boardForRendering.map((row, rowIndex) => {
        const rank = 1 + (shouldRotate ? rowIndex : numRanks - rowIndex - 1);

        return (
          <div key={rowIndex} className="board-row">
            {row.map((piece, colIndex) => {
              const file =
                1 + (shouldRotate ? numFiles - colIndex - 1 : colIndex);

              const [positionLabel, isBottomLeftMost] = getPositionLabel(
                rank,
                file
              );

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
                    isViewingLatestBoard &&
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
                    <span
                      className={`rank-file-label${
                        isBottomLeftMost ? " rank-file-label--corner" : ""
                      }`}
                    >
                      {positionLabel}
                    </span>
                  }
                  {piece ? (
                    <img
                      className="piece"
                      src={imageMap[piece.pieceType][piece.color]}
                      alt={`${piece.color} ${piece.pieceType}`}
                      data-rank={rank}
                      data-file={file}
                      onDragStart={(e) => {
                        handleDragStart(e, piece);
                      }}
                      onTouchMove={(e) => {
                        handleDragStart(e, piece);
                      }}
                    />
                  ) : (
                    <img
                      className="hidden-piece"
                      src={imageMap[PieceType.Pawn][Color.White]}
                      data-rank={rank}
                      data-file={file}
                      style={{
                        visibility: "hidden",
                        opacity: 0,
                      }}
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
