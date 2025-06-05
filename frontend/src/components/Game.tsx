import {
  GameEndingCheckmate,
  GameEndingType,
  GameRecord,
  GameStateType,
} from "../types/game";
import { Color, getOppositePlayerColor } from "../types/piece";
import { ChessBoard } from "./chess-board/ChessBoard";
import { Alert } from "./Alert";
import { CapturedPieces } from "./CapturedPieces";
import { GameMessage } from "../types/sharedComponentTypes";
import { GameRequest } from "../types/api";
import { useMemo, useState, useEffect } from "react";
import { capitalizeFirstLetter, getLast } from "../utils";

import "../css/Game.css";

interface GameProps {
  gameRecord: GameRecord;
  onHideGame: (gameId: string) => void;
  connectionId: string;
  messages: GameMessage[];
  sendWebSocketMessage: (action: GameRequest) => void;
  dismissMessage: (id: string) => void;
}

export const Game: React.FC<GameProps> = ({
  gameRecord,
  onHideGame,
  connectionId,
  messages,
  sendWebSocketMessage,
  dismissMessage,
}) => {
  const gameState = gameRecord.game_state;
  const currentGameState = getLast(gameState.history);
  const gameStateType = currentGameState.state;
  const numStates = gameState.history.length;

  const [historyIndex, setHistoryIndex] = useState(numStates - 1);

  // Reset to latest board when game state updates
  useEffect(() => {
    setHistoryIndex(numStates - 1);
  }, [numStates]);

  const bothPlayersReady = ![
    gameRecord.black_connection_id ?? "<disconnected>",
    gameRecord.white_connection_id ?? "<disconnected>",
  ].includes("<disconnected>");

  const gameIsInProgress = gameStateType === GameStateType.InProgress;
  const gameIsFinished = typeof gameStateType === "object";

  const playerColor =
    connectionId === gameRecord.white_connection_id ? Color.White : Color.Black;

  const isTurn = playerColor === currentGameState.currentTurn;

  const viewedGameState = gameState.history[historyIndex];

  const playerCapturedPieces = viewedGameState.capturedPieces[playerColor];

  const playerPointsLead =
    playerColor === Color.White
      ? viewedGameState.capturedPieces.whitePoints -
        viewedGameState.capturedPieces.blackPoints
      : viewedGameState.capturedPieces.blackPoints -
        viewedGameState.capturedPieces.whitePoints;

  const opponentPointsLead =
    playerColor === Color.White
      ? viewedGameState.capturedPieces.blackPoints -
        viewedGameState.capturedPieces.whitePoints
      : viewedGameState.capturedPieces.whitePoints -
        viewedGameState.capturedPieces.blackPoints;

  const opponentCapturedPieces =
    viewedGameState.capturedPieces[getOppositePlayerColor(playerColor)];

  const stateOfGame = useMemo(() => {
    if (gameIsInProgress) {
      return currentGameState.inCheck
        ? [
            `${
              currentGameState.inCheck === Color.White ? "White" : "Black"
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
  }, [gameIsInProgress, currentGameState.inCheck, gameStateType, playerColor]);

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
      <div className="game-id-container">
        <h2 className="game-id">Game: {gameRecord.game_id}</h2>

        <button
          className="leave-game-button"
          onClick={() => {
            onHideGame(gameRecord.game_id);
          }}
        >
          Leave game
        </button>
      </div>

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
            gameState={gameState}
            playerColor={playerColor}
            gameId={gameRecord.game_id}
            sendWebSocketMessage={sendWebSocketMessage}
            historyIndex={historyIndex}
          />
        </div>

        <div className="board-history-controls">
          <button
            disabled={historyIndex === 0}
            onClick={() => {
              setHistoryIndex((prev) => Math.max(0, prev - 1));
            }}
          >
            &lt; Previous
          </button>
          <span>
            State {historyIndex + 1} of {numStates}
          </span>
          <button
            disabled={historyIndex === numStates - 1}
            onClick={() => {
              setHistoryIndex((prev) => Math.min(numStates - 1, prev + 1));
            }}
          >
            Next &gt;
          </button>
        </div>

        <CapturedPieces
          pieces={playerCapturedPieces}
          pointsLead={playerPointsLead}
        />
      </div>
    </div>
  );
};
