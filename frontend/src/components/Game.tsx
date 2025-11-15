import { useMemo, useState, useEffect, useRef } from "react";

import {
  capitalizeFirstLetter,
  getCapturedPiecesFromBase64,
  getLast,
} from "@src/utils";

import { Alert } from "@src/components/Alert";
import { BoardHistoryControls } from "@src/components/BoardHistoryControls";
import { CapturedPieces } from "@src/components/CapturedPieces";
import { Color, getOppositePlayerColor } from "@src/types/piece";
import { ChessBoard } from "@src/components/chess-board/ChessBoard";
import { PlayerTime } from "@src/components/PlayerTime";

import { useNotifications } from "@src/hooks/useNotifications";
import { useTitleAnimation } from "@src/hooks/useTitleAnimation";

import { getSquaresFromCompactBoard } from "@src/utils";
import { ExpandedGameStateAtPointInTime } from "@src/types/board";
import { GameRequest } from "@src/types/api";
import {
  GameEndingCheckmate,
  GameEndingOutOfTime,
  GameEndingResignation,
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
  onLeaveGame: (gameId: string) => void;
  connectionId: string;
  messages: GameMessage[];
  sendWebSocketMessage: (action: GameRequest) => void;
  dismissMessage: (id: string) => void;
  totalActiveGames: number;
}

export const Game: React.FC<GameProps> = ({
  gameRecord,
  onLeaveGame,
  connectionId,
  messages,
  sendWebSocketMessage,
  dismissMessage,
  totalActiveGames,
}) => {
  const gameId = gameRecord.game_id;
  const [showResignConfirm, setShowResignConfirm] = useState(false);
  const { requestPermission, showNotification } = useNotifications();
  const previousIsTurnRef = useRef<boolean | null>(null);

  const gameState = gameRecord.game_state;

  const history: ExpandedGameStateAtPointInTime[] = useMemo(
    () =>
      gameState.history.map((state) => ({
        ...state,
        board: { squares: getSquaresFromCompactBoard(state.board) },
      })),
    [gameState.history]
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
  const isActivePlayerTurn = isTurn && gameIsInProgress && bothPlayersReady;

  useTitleAnimation(isActivePlayerTurn, "♟️ Your turn!");

  const viewedGameState = history[historyIndex];

  const expandedCapturedPieces = useMemo(
    () => getCapturedPiecesFromBase64(viewedGameState.capturedPieces),
    [viewedGameState.capturedPieces]
  );

  useEffect(() => {
    const justBecameMyTurn =
      previousIsTurnRef.current === false && isActivePlayerTurn;

    if (justBecameMyTurn && totalActiveGames <= 3) {
      void requestPermission().then((granted) => {
        if (granted) {
          const notification = showNotification("Your turn!", {
            body: `It's your turn in game ${gameId}`,
            tag: `turn-${gameId}`,
            requireInteraction: false,
          });

          if (notification) {
            setTimeout(() => {
              notification.close();
            }, 5000);

            notification.onclick = () => {
              window.focus();
              notification.close();
            };
          }
        }
      });
    }

    previousIsTurnRef.current = isTurn;
  }, [
    isActivePlayerTurn,
    gameId,
    totalActiveGames,
    requestPermission,
    showNotification,
    isTurn,
  ]);

  const playerCapturedPieces = expandedCapturedPieces[playerColor];

  const playerPointsLead =
    playerColor === Color.White
      ? expandedCapturedPieces.whitePoints - expandedCapturedPieces.blackPoints
      : expandedCapturedPieces.blackPoints - expandedCapturedPieces.whitePoints;

  const opponentPointsLead =
    playerColor === Color.White
      ? expandedCapturedPieces.blackPoints - expandedCapturedPieces.whitePoints
      : expandedCapturedPieces.whitePoints - expandedCapturedPieces.blackPoints;

  const opponentCapturedPieces = expandedCapturedPieces[opponentColor];

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

      if (gameEndingType === GameEndingType.Resignation) {
        const losingColor = (gameEnding as GameEndingResignation)[
          gameEndingType
        ];

        return [
          `${capitalizeFirstLetter(losingColor)} resigned!`,
          playerColor === losingColor ? "red" : "green",
        ];
      }

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

  const handleResign = () => {
    sendWebSocketMessage({
      route: API_ROUTE,
      data: {
        [PlayerActionName.Resign]: {
          gameId,
        },
      },
    });

    setShowResignConfirm(false);
  };

  return (
    <div id={`game-${gameId}`} className="game-container">
      <div className="game-id-container">
        <h2 className="game-id">Game: {gameId}</h2>

        <div className="game-buttons">
          {gameIsInProgress && bothPlayersReady && !gameIsFinished && (
            <button
              className="resign-button"
              onClick={() => {
                setShowResignConfirm(true);
              }}
            >
              Resign
            </button>
          )}
          <button
            className="leave-game-button"
            onClick={() => {
              onLeaveGame(gameId);
            }}
          >
            Leave game
          </button>
        </div>
      </div>

      {showResignConfirm && (
        <div className="resign-confirm">
          <div className="resign-confirm-inner">
            <p>Are you sure you want to resign?</p>
            <div className="resign-confirm-buttons">
              <button className="resign-confirm-yes" onClick={handleResign}>
                Yes, resign
              </button>
              <button
                className="resign-confirm-no"
                onClick={() => {
                  setShowResignConfirm(false);
                }}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

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
          className={`chess-board-container${isTurn ? " is-player-turn" : ""}`}
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
