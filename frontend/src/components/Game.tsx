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

import { useLocalStorage } from "@src/hooks/useLocalStorage";
import { useNotifications } from "@src/hooks/useNotifications";
import { useTitleAnimation } from "@src/hooks/useTitleAnimation";

import { getSquaresFromCompactBoard } from "@src/utils";
import { ExpandedGameStateAtPointInTime } from "@src/types/board";
import { GameRequest } from "@src/types/api";
import {
  AiAnalysisResult,
  AnalysisType,
  GameEndingCheckmate,
  GameEndingOutOfTime,
  GameEndingResignation,
  GameEndingType,
  GameRecord,
  GameStateType,
  PlayerActionName,
} from "@src/types/game";
import { GameMessage } from "@src/types/sharedComponentTypes";

import { API_ROUTE, BOARD_THEMES, BoardTheme } from "@src/constants";

import "@src/css/Game.css";

interface GameProps {
  gameRecord: GameRecord;
  onLeaveGame: (gameId: string) => void;
  onPlayAgain: (gameId: string) => void;
  connectionId: string;
  messages: GameMessage[];
  sendWebSocketMessage: (action: GameRequest) => void;
  dismissMessage: (id: string) => void;
  totalActiveGames: number;
  aiAnalysis: AiAnalysisResult | null;
  onRequestAiAnalysis: (gameId: string) => void;
  onClearAiAnalysis: (gameId: string) => void;
}

export const Game: React.FC<GameProps> = ({
  gameRecord,
  onLeaveGame,
  onPlayAgain,
  connectionId,
  messages,
  sendWebSocketMessage,
  dismissMessage,
  totalActiveGames,
  aiAnalysis,
  onRequestAiAnalysis,
  onClearAiAnalysis,
}) => {
  const gameId = gameRecord.game_id;
  const [showResignConfirm, setShowResignConfirm] = useState(false);
  const [aiLoading, setAiLoading] = useState<AnalysisType | null>(null);
  const { requestPermission, showNotification } = useNotifications();
  const previousIsTurnRef = useRef<boolean | null>(null);

  const [boardThemeId, setBoardThemeId] = useLocalStorage(
    "board-theme",
    "classic",
  );
  const boardTheme: BoardTheme =
    BOARD_THEMES.find((theme) => theme.id === boardThemeId) ?? BOARD_THEMES[0];

  const gameState = gameRecord.game_state;

  const history: ExpandedGameStateAtPointInTime[] = useMemo(
    () =>
      gameState.history.map((state) => ({
        ...state,
        board: { squares: getSquaresFromCompactBoard(state.board) },
      })),
    [gameState.history],
  );

  const gameTime = gameState.gameTime;

  const currentGameState = getLast(history);
  const gameStateType = currentGameState.state;
  const numStates = history.length;

  const [historyIndex, setHistoryIndex] = useState(numStates - 1);

  const bothPlayersReady =
    gameRecord.engine_difficulty !== null ||
    ![
      gameRecord.black_connection_id ?? "<disconnected>",
      gameRecord.white_connection_id ?? "<disconnected>",
    ].includes("<disconnected>");

  const gameIsInProgress = gameStateType === GameStateType.InProgress;
  const gameIsFinished = typeof gameStateType === "object";

  const playerColor =
    connectionId === gameRecord.white_connection_id ? Color.White : Color.Black;

  const opponentColor = getOppositePlayerColor(playerColor);

  const isTurn = playerColor === currentGameState.currentTurn;

  const isActivePlayerTurn = useMemo(
    () => isTurn && gameIsInProgress && bothPlayersReady,
    [isTurn, gameIsInProgress, bothPlayersReady],
  );

  const [isDocumentHidden, setIsDocumentHidden] = useState(document.hidden);

  useEffect(() => {
    const handleVisibilityChange = () => {
      setIsDocumentHidden(document.hidden);
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, []);

  useTitleAnimation(isActivePlayerTurn && isDocumentHidden, "♟️ Your turn!");

  const viewedGameState = history[historyIndex];

  const expandedCapturedPieces = useMemo(
    () => getCapturedPiecesFromBase64(viewedGameState.capturedPieces),
    [viewedGameState.capturedPieces],
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
      : null,
  );

  const [opponentSecondsLeft, setOpponentSecondsLeft] = useState(
    gameIsTimed
      ? playerColor === Color.White
        ? gameTime.blackSecondsLeft
        : gameTime.whiteSecondsLeft
      : null,
  );

  // Reset to latest board when game state updates
  useEffect(() => {
    setHistoryIndex(numStates - 1);

    if (
      gameIsTimed &&
      gameRecord.engine_difficulty !== null &&
      currentGameState.engineResult
    ) {
      const searchTimeMs = currentGameState.engineResult.timeMs;

      setOpponentSecondsLeft((prev) => {
        if (prev === null) {
          return prev;
        }

        return prev - Math.max(1, Math.floor(searchTimeMs / 1000));
      });
    }
  }, [
    currentGameState.engineResult,
    gameIsTimed,
    gameRecord.engine_difficulty,
    numStates,
  ]);

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

  useEffect(() => {
    if (typeof gameStateType === "object") {
      const gameEnding = gameStateType[GameStateType.Finished];

      if (typeof gameEnding === "object") {
        const gameEndingType = Object.keys(gameEnding)[0] as GameEndingType;
        if (gameEndingType === GameEndingType.Resignation) {
          setHistoryIndex(numStates - 1);
        }
      }
    }
  }, [gameStateType, numStates]);

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
          (gameEnding as GameEndingCheckmate)[gameEndingType],
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

  const opponentUsername = gameRecord.engine_difficulty
    ? "Engine"
    : playerColor === Color.White
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

  const handlePlayAgain = () => {
    const username =
      playerColor === Color.White
        ? gameRecord.white_username
        : gameRecord.black_username;

    if (!username) return;

    sendWebSocketMessage({
      route: API_ROUTE,
      data: {
        [PlayerActionName.CreateGame]: {
          username,
          gameId: null,
          boardSetup: gameRecord.board_setup,
          colorPreference: gameRecord.color_preference,
          secondsPerPlayer: gameRecord.seconds_per_player,
          engineDifficulty: gameRecord.engine_difficulty,
        },
      },
    });

    onPlayAgain(gameId);
  };

  useEffect(() => {
    if (aiAnalysis) {
      setAiLoading(null);
    }
  }, [aiAnalysis]);

  const handleAiAnalysis = (analysisType: AnalysisType) => {
    setAiLoading(analysisType);
    onClearAiAnalysis(gameId);
    onRequestAiAnalysis(gameId);
    sendWebSocketMessage({
      route: API_ROUTE,
      data: {
        [PlayerActionName.AnalyzePosition]: {
          gameId,
          analysisType,
        },
      },
    });
  };

  const AI_BUTTONS: { type: AnalysisType; label: string }[] = [
    { type: AnalysisType.MoveExplanation, label: "Explain" },
    { type: AnalysisType.BlunderDetection, label: "Blunder?" },
    { type: AnalysisType.Coach, label: "Coach" },
    { type: AnalysisType.PostGame, label: "Post-game" },
  ];

  const hasMovesPlayed = numStates > 1;

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
              <button
                className="resign-confirm-no"
                onClick={() => {
                  setShowResignConfirm(false);
                }}
              >
                Cancel
              </button>
              <button className="resign-confirm-yes" onClick={handleResign}>
                Yes, resign
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
                {bothPlayersReady || gameRecord.engine_difficulty
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

            {gameRecord.engine_difficulty && viewedGameState.engineResult && (
              <p className="pill pill--blue">
                {viewedGameState.engineResult.fromBook ? (
                  "Book move"
                ) : (
                  <>
                    D{viewedGameState.engineResult.depth} ·{" "}
                    {(
                      (viewedGameState.engineResult.nodes -
                        viewedGameState.engineResult.qnodes) /
                      1000
                    ).toFixed(1)}
                    k N ·{" "}
                    {(viewedGameState.engineResult.qnodes / 1000).toFixed(1)}k
                    QN · {viewedGameState.engineResult.timeMs}ms
                  </>
                )}
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
        </div>

        <div className="board-with-theme-picker">
          <div>
            {gameIsTimed && (
              <PlayerTime
                secondsLeft={
                  playerOutOfTime === opponentColor
                    ? 0
                    : (opponentSecondsLeft ?? 0)
                }
              />
            )}

            <div
              className={`chess-board-container${
                isTurn ? " is-player-turn" : ""
              }`}
              style={
                {
                  "--board-border-color": boardTheme.borderColor,
                } as React.CSSProperties
              }
            >
              <ChessBoard
                expandedHistory={history}
                playerColor={playerColor}
                gameId={gameId}
                sendWebSocketMessage={sendWebSocketMessage}
                historyIndex={historyIndex}
                isViewingLatestBoard={isViewingLatestBoard}
                gameOverMessage={gameOverMessage}
                isTurn={isTurn}
                onPlayAgain={handlePlayAgain}
                boardTheme={boardTheme}
              />
            </div>

            {gameIsTimed && <PlayerTime secondsLeft={playerSecondsLeft ?? 0} />}
          </div>

          <div className="board-theme-picker">
            {BOARD_THEMES.map((theme) => (
              <button
                key={theme.id}
                className={`theme-swatch${
                  boardTheme.id === theme.id ? " theme-swatch--active" : ""
                }`}
                style={{
                  background: `linear-gradient(135deg, ${
                    theme.lightColor
                  } 50%, ${theme.darkColor} 50%)`,
                }}
                title={theme.label}
                onClick={() => {
                  setBoardThemeId(theme.id);
                }}
              />
            ))}
          </div>
        </div>

        <div className="player-status-row">
          <CapturedPieces
            pieces={playerCapturedPieces}
            pointsLead={playerPointsLead}
          />
        </div>

        <BoardHistoryControls
          historyIndex={historyIndex}
          setHistoryIndex={setHistoryIndex}
          numStates={numStates}
        />

        {hasMovesPlayed && (
          <div className="ai-analysis-section">
            <div className="ai-analysis-buttons">
              {AI_BUTTONS.filter(
                ({ type }) => type !== AnalysisType.PostGame || gameIsFinished,
              ).map(({ type, label }) => (
                <button
                  key={type}
                  className={`ai-analysis-button${
                    aiAnalysis?.analysisType === type
                      ? " ai-analysis-button--active"
                      : ""
                  }`}
                  disabled={aiLoading !== null}
                  onClick={() => {
                    handleAiAnalysis(type);
                  }}
                >
                  {aiLoading === type ? "Analyzing..." : label}
                </button>
              ))}
            </div>

            {(aiLoading ?? aiAnalysis) && (
              <div className="ai-analysis-result">
                {aiAnalysis && (
                  <button
                    className="ai-analysis-clear"
                    onClick={() => {
                      onClearAiAnalysis(gameId);
                    }}
                    aria-label="Clear analysis"
                  >
                    ✕
                  </button>
                )}
                {aiLoading && !aiAnalysis && (
                  <p className="ai-analysis-loading">Thinking...</p>
                )}
                {aiAnalysis && (
                  <p className="ai-analysis-text">{aiAnalysis.text}</p>
                )}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
