import { useMemo, useState, useEffect } from "react";

import { capitalizeFirstLetter, getLast } from "@src/utils";

import { Alert } from "@src/components/Alert";
import { BoardHistoryControls } from "@src/components/BoardHistoryControls";
import { CapturedPieces } from "@src/components/CapturedPieces";
import { Color, getOppositePlayerColor } from "@src/types/piece";
import { ChessBoard } from "@src/components/chess-board/ChessBoard";
import { PlayerTime } from "@src/components/PlayerTime";

import { getBoardFromBase64 } from "@src/utils";
import { ExpandedGameStateAtPointInTime } from "@src/types/board";
import { GameRequest } from "@src/types/api";
import {
  GameEndingCheckmate,
  GameEndingOutOfTime,
  GameEndingType,
  GameRecord,
  GameStateType,
  PlayerActionName,
} from "@src/types/game";
import { GameMessage } from "@src/types/sharedComponentTypes";

import { API_ROUTE } from "@src/constants";

import "@src/css/Game.css";

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
  const gameId = gameRecord.game_id;

  const gameState = gameRecord.game_state;

  const history: ExpandedGameStateAtPointInTime[] = gameState.history.map(
    (state) => ({
      ...state,
      board: { squares: getBoardFromBase64(state.board) },
    })
  );

  const gameTime = gameState.gameTime;

  const currentGameState = getLast(history);
  const gameStateType = currentGameState.state;
  const numStates = history.length;

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

  const opponentColor = getOppositePlayerColor(playerColor);

  const isTurn = playerColor === currentGameState.currentTurn;

  const viewedGameState = history[historyIndex];

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

  const opponentCapturedPieces = viewedGameState.capturedPieces[opponentColor];

  const gameIsTimed = gameTime !== null;

  const [playerSecondsLeft, setPlayerSecondsLeft] = useState(
    gameIsTimed
      ? playerColor === Color.White
        ? gameTime.whiteSecondsLeft
        : gameTime.blackSecondsLeft
      : null
  );

  const [opponentSecondsLeft, setOpponentSecondsLeft] = useState(
    gameIsTimed
      ? playerColor === Color.White
        ? gameTime.blackSecondsLeft
        : gameTime.whiteSecondsLeft
      : null
  );

  useEffect(() => {
    if (!gameIsTimed || !bothPlayersReady || gameIsFinished) {
      return;
    }

    const intervalId = setInterval(() => {
      if (playerSecondsLeft === 0 || opponentSecondsLeft === 0) {
        return;
      }

      (isTurn ? setPlayerSecondsLeft : setOpponentSecondsLeft)((old) => {
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        const newValue = old! - 1;

        if (newValue === 0) {
          if (isTurn) {
            sendWebSocketMessage({
              route: API_ROUTE,
              data: {
                [PlayerActionName.LoseViaOutOfTime]: {
                  gameId,
                },
              },
            });
          }

          clearInterval(intervalId);
        }

        return newValue;
      });
    }, 1000);

    return () => {
      clearInterval(intervalId);
    };
  }, [
    bothPlayersReady,
    gameId,
    gameIsFinished,
    gameIsInProgress,
    gameIsTimed,
    isTurn,
    opponentSecondsLeft,
    playerSecondsLeft,
    sendWebSocketMessage,
  ]);

  const [playerOutOfTime, setPlayerOutOfTime] = useState<Color | null>(null);

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

      if (gameEndingType === GameEndingType.OutOfTime) {
        const losingColor = (gameEnding as GameEndingOutOfTime)[gameEndingType];
        const winningColor = getOppositePlayerColor(losingColor);
        setPlayerOutOfTime(losingColor);

        return playerColor === winningColor
          ? [
              `${capitalizeFirstLetter(losingColor)} is out of time! You win!`,
              "green",
            ]
          : [
              `Out of time! ${capitalizeFirstLetter(winningColor)} wins!`,
              "red",
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

  const isViewingLatestBoard = historyIndex === numStates - 1;

  const gameOverMessage =
    gameIsFinished && isViewingLatestBoard ? stateOfGame[0] : null;

  return (
    <div id={`game-${gameId}`} className="game-container">
      <div className="game-id-container">
        <h2 className="game-id">Game: {gameId}</h2>

        <button
          className="leave-game-button"
          onClick={() => {
            onHideGame(gameId);
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
                  ? "YOUR TURN"
                  : `${playerColor === Color.White ? "Black" : "White"}'s turn`}
              </p>
            )}
          </>
        )}
      </div>

      <div className="game-area">
        <div className="player-status-row">
          <CapturedPieces
            pieces={opponentCapturedPieces}
            pointsLead={opponentPointsLead}
          />

          {gameIsTimed && (
            <PlayerTime
              secondsLeft={
                playerOutOfTime === opponentColor ? 0 : opponentSecondsLeft ?? 0
              }
            />
          )}
        </div>

        <div
          className={`chess-board-container ${isTurn ? "is-player-turn" : ""}`}
        >
          <ChessBoard
            expandedHistory={history}
            playerColor={playerColor}
            gameId={gameId}
            sendWebSocketMessage={sendWebSocketMessage}
            historyIndex={historyIndex}
            isViewingLatestBoard={isViewingLatestBoard}
            gameOverMessage={gameOverMessage}
          />
        </div>

        <div className="player-status-row">
          <CapturedPieces
            pieces={playerCapturedPieces}
            pointsLead={playerPointsLead}
          />

          {gameIsTimed && <PlayerTime secondsLeft={playerSecondsLeft ?? 0} />}
        </div>

        <BoardHistoryControls
          historyIndex={historyIndex}
          setHistoryIndex={setHistoryIndex}
          numStates={numStates}
        />
      </div>
    </div>
  );
};
