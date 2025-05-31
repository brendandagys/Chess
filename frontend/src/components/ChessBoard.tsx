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
}

export const ChessBoard: React.FC<ChessBoardProps> = ({
  gameState,
  playerColor,
  gameId,
  sendWebSocketMessage,
}) => {
  const shouldRotate = playerColor === Color.Black;
  const board = shouldRotate
    ? rotateMatrix180Degrees(gameState.board.squares)
    : gameState.board.squares;

  const [selectedSquare, setSelectedSquare] = useState<Position | null>(null);

  const [draggingPiece, handleDragStart] = useDrag(
    gameId,
    sendWebSocketMessage
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

    const numRanks = one.squares.length;

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
                <div
                  key={colIndex}
                  className={`square${
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
