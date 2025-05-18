import {
  GameEndingCheckmate,
  GameEndingType,
  GameRecord,
  GameStateType,
} from "../types/game";
import { Color, getOppositePlayerColor } from "../types/piece";
import { ChessBoard } from "./ChessBoard";
import { Alert } from "./Alert";
import { CapturedPieces } from "./CapturedPieces";
import { GameMessage } from "../types/sharedComponentTypes";
import { GameRequest } from "../types/api";
import { useMemo } from "react";
import { capitalizeFirstLetter } from "../utils";

import "../css/Game.css";

interface GameProps {
  gameRecord: GameRecord;
  connectionId: string;
  messages: GameMessage[];
  sendWebSocketMessage: (action: GameRequest) => void;
  dismissMessage: (id: string) => void;
}

export const Game: React.FC<GameProps> = ({
  gameRecord,
  connectionId,
  messages,
  sendWebSocketMessage,
  dismissMessage,
}) => {
  const gameState = gameRecord.game_state;
  const gameStateType = gameState.state;

  const bothPlayersReady = ![
    gameRecord.black_connection_id ?? "<disconnected>",
    gameRecord.white_connection_id ?? "<disconnected>",
  ].includes("<disconnected>");

  const gameIsInProgress = gameStateType === GameStateType.InProgress;
  const gameIsFinished = typeof gameStateType === "object";

  const playerColor =
    connectionId === gameRecord.white_connection_id ? Color.White : Color.Black;

  const isTurn = playerColor === gameState.currentTurn;

  const playerCapturedPieces = gameState.capturedPieces[playerColor];

  const playerPointsLead =
    playerColor === Color.White
      ? gameState.capturedPieces.whitePoints -
        gameState.capturedPieces.blackPoints
      : gameState.capturedPieces.blackPoints -
        gameState.capturedPieces.whitePoints;

  const opponentPointsLead =
    playerColor === Color.White
      ? gameState.capturedPieces.blackPoints -
        gameState.capturedPieces.whitePoints
      : gameState.capturedPieces.whitePoints -
        gameState.capturedPieces.blackPoints;

  const opponentCapturedPieces =
    gameState.capturedPieces[getOppositePlayerColor(playerColor)];

  const stateOfGame = useMemo(() => {
    if (gameIsInProgress) {
      return gameState.inCheck
        ? [
            `${
              gameState.inCheck === Color.White ? "White" : "Black"
            } is in check!`,
            "red",
          ]
        : ["Game in progress", "blue"];
    }

    if (gameStateType === GameStateType.NotStarted) {
      return ["Game not started", "red"];
    }

    const gameEnding = gameStateType[GameStateType.Finished];

    if (typeof gameEnding === "object") {
      const gameEndingType = Object.keys(gameEnding)[0] as GameEndingType;

      if (gameEndingType === GameEndingType.Checkmate) {
        const winningColor = getOppositePlayerColor(
          (gameEnding as GameEndingCheckmate)[gameEndingType]
        );

        return [
          `Checkmate! ${capitalizeFirstLetter(winningColor)} wins!`,
          playerColor === winningColor ? "green" : "red",
        ];
      }
    }

    return ["Game over", "gray"];
  }, [gameIsInProgress, gameState.inCheck, gameStateType, playerColor]);

  const playerUsername =
    playerColor === Color.White
      ? gameRecord.white_username
      : gameRecord.black_username;

  const opponentUsername =
    playerColor === Color.White
      ? gameRecord.black_username
      : gameRecord.white_username;

  return (
    <div className="game-container">
      <h2 className="game-id">Game: {gameRecord.game_id}</h2>

      <div className="status-container">
        {messages.length ? (
          messages.map((message) => (
            <Alert
              key={message.id}
              message={message}
              onDismiss={() => {
                dismissMessage(message.id);
              }}
            />
          ))
        ) : (
          <>
            <p className={`pill pill--${stateOfGame[1]}`}>{stateOfGame[0]}</p>

            {!gameIsFinished && (
              <p className="pill pill--gray">
                {bothPlayersReady
                  ? `${playerUsername} vs. ${opponentUsername}`
                  : `Waiting for ${opponentUsername ?? "other player"}...`}
              </p>
            )}

            {gameIsInProgress && bothPlayersReady && (
              <p className={`pill pill--green ${!isTurn ? "pill--faded" : ""}`}>
                {isTurn
                  ? "Your turn!"
                  : `${playerColor === Color.White ? "Black" : "White"}'s turn`}
              </p>
            )}
          </>
        )}
      </div>

      <div className="game-area">
        <CapturedPieces
          pieces={opponentCapturedPieces}
          pointsLead={opponentPointsLead}
        />

        <div
          className={`chess-board-container ${isTurn ? "is-player-turn" : ""}`}
        >
          <ChessBoard
            board={gameState.board}
            playerColor={playerColor}
            gameId={gameRecord.game_id}
            sendWebSocketMessage={sendWebSocketMessage}
          />
        </div>

        <CapturedPieces
          pieces={playerCapturedPieces}
          pointsLead={playerPointsLead}
        />
      </div>
    </div>
  );
};
