import { useState, useCallback, useEffect, useRef } from "react";

import { useNav } from "@src/context/useNav";
import { useLocalStorage } from "@src/hooks/useLocalStorage";
import { useMessageDisplay } from "@src/hooks/useMessageDisplay";
import { useScroll } from "@src/hooks/useScroll";
import { useWebSocket } from "@src/hooks/useWebSocket";

import { Alert } from "@src/components/Alert";
import { MenuButtons } from "@src/components/MenuButtons";
import { HeaderSection } from "@src/components/HeaderSection";
import { GameForm } from "@src/components/GameForm";
import { Game } from "@src/components/Game";

import { ApiMessageType, ApiResponse } from "@src/types/api";
import {
  AiAnalysisResult,
  GameRecord,
  PlayerActionName,
} from "@src/types/game";
import { FormToShow } from "@src/types/sharedComponentTypes";
import { API_ROUTE, WEBSOCKET_ENDPOINT } from "@src/constants";

import { playIllegalSound } from "@src/sounds";
import "@src/css/App.css";

// Approximate normal distribution via Box-Muller: mean ~2500ms, std ~500ms,
// clamped to [1000, 4000]ms so the engine appears to "think" for 1-4 seconds.
function getRealisticDelayMs(): number {
  const u1 = Math.random();
  const u2 = Math.random();
  const z =
    Math.sqrt(-2 * Math.log(u1 === 0 ? Number.EPSILON : u1)) *
    Math.cos(2 * Math.PI * u2);
  return Math.round(Math.max(1000, Math.min(6000, 2500 + z * 500)));
}

export const App: React.FC = () => {
  const {
    username,
    gameIds,
    addGameId: addGameIdToURL,
    removeGameId,
    setUsername,
  } = useNav();

  const [gameIdsFromUrl] = useState([...gameIds]);
  const [usernameFromLocalStorage] = useLocalStorage("username", "");
  const [gameRecords, setGameRecords] = useState<GameRecord[]>([]);

  const [realismPref, setRealismPref] = useLocalStorage(
    "realism-enabled",
    "true",
  );
  const realismOn = realismPref !== "false";

  const [pvPref, setPvPref] = useLocalStorage("pv-enabled", "true");
  const pvOn = pvPref !== "false";

  const gameRecordsRef = useRef<GameRecord[]>([]);
  useEffect(() => {
    gameRecordsRef.current = gameRecords;
  }, [gameRecords]);

  const realismPrefRef = useRef(realismPref);
  useEffect(() => {
    realismPrefRef.current = realismPref;
  }, [realismPref]);

  const [appMessages, setAppMessages, dismissAppMessage] = useMessageDisplay();
  const [gameMessages, setGameMessages, dismissGameMessage] =
    useMessageDisplay();

  const [aiAnalyses, setAiAnalyses] = useState<
    Record<string, AiAnalysisResult>
  >({});
  const pendingAiGameIdRef = useRef<string | null>(null);
  const pendingPlayAgainIndexRef = useRef<number | null>(null);

  const [formToShow, setFormToShow] = useState<FormToShow>(
    !username && gameIds.length ? FormToShow.Join : FormToShow.Create,
  );
  const [showForm, setShowForm] = useState(
    !/^\/game\/(.+)$/.exec(window.location.pathname) ||
      !usernameFromLocalStorage,
  );

  const scrollTo = useScroll();

  const onWebSocketMessage = useCallback(
    (response: ApiResponse<GameRecord | AiAnalysisResult | null>) => {
      const isGameRecord =
        response.data && Object.keys(response.data).includes("game_id");

      const isAiAnalysis =
        response.data &&
        Object.keys(response.data).includes("analysisType") &&
        Object.keys(response.data).includes("text");

      const gameRecord = isGameRecord ? (response.data as GameRecord) : null;

      if (isAiAnalysis) {
        const result = response.data as AiAnalysisResult;
        const gameId = pendingAiGameIdRef.current;

        if (gameId) {
          setAiAnalyses((old) => ({ ...old, [gameId]: result }));
          pendingAiGameIdRef.current = null;
        }
      }

      if (gameRecord) {
        const playAgainIndex = pendingPlayAgainIndexRef.current;

        if (playAgainIndex !== null) {
          pendingPlayAgainIndexRef.current = null;
        }

        // SIMULATED ENGINE DELAY LOGIC
        const isEngineGame = gameRecord.engine_difficulty !== null;
        const existingRecord = gameRecordsRef.current.find(
          (g) => g.game_id === gameRecord.game_id,
        );
        const historyGrowth =
          gameRecord.game_state.history.length -
          (existingRecord?.game_state.history.length ?? 0);

        // The server sends two messages per engine turn: one after the player
        // moves (engineResult = null) and one after the engine responds
        // (engineResult set). Delay only the second message.
        const latestEngineResult =
          gameRecord.game_state.history[
            gameRecord.game_state.history.length - 1
          ].engineResult;

        const shouldDelay =
          isEngineGame &&
          realismPrefRef.current !== "false" &&
          existingRecord !== undefined &&
          historyGrowth > 0 &&
          latestEngineResult !== null;

        if (shouldDelay) {
          const trimmedRecord: GameRecord = {
            ...gameRecord,
            game_state: {
              ...gameRecord.game_state,
              history: gameRecord.game_state.history.slice(0, -1),
            },
          };

          const replaceRecord = (record: GameRecord) => {
            setGameRecords((old) => {
              const index = old.findIndex(
                (game) => game.game_id === record.game_id,
              );
              if (index === -1) return old;
              const newGames = [...old];
              newGames[index] = record;
              return newGames;
            });
          };

          replaceRecord(trimmedRecord);

          setTimeout(() => {
            replaceRecord(gameRecord);
          }, getRealisticDelayMs());
        } else {
          setGameRecords((old) => {
            const index = old.findIndex(
              (game) => game.game_id === gameRecord.game_id,
            );

            if (index === -1) {
              if (playAgainIndex !== null) {
                removeGameId(gameRecord.game_id); // Remove from URL
                const newGames = [...old];
                newGames[playAgainIndex] = gameRecord;
                return newGames;
              }

              setTimeout(() => {
                scrollTo(`game-${gameRecord.game_id}`);
              }, 100);

              return [...old, gameRecord];
            }

            const newGames = [...old];
            newGames[index] = gameRecord;

            const moveMade =
              gameRecord.game_state.history.length !==
              old[index].game_state.history.length;

            if (!moveMade) {
              document.querySelectorAll(".dragging").forEach((el) => {
                el.classList.remove("dragging");
              });
            }

            return newGames;
          });
        }
      }

      if (response.messages.length) {
        if (
          response.messages.some((m) => m.messageType === ApiMessageType.Error)
        ) {
          playIllegalSound();
        }

        (gameRecord ? setGameMessages : setAppMessages)((old) => [
          ...old.filter(
            (o) =>
              !response.messages.some(
                (m) =>
                  m.message === o.message && m.messageType === o.messageType,
              ),
          ),
          ...response.messages.map(({ message, messageType }) => ({
            id: `${
              gameRecord?.game_id ? `${gameRecord.game_id}-` : ""
            }${Math.random().toString(36).slice(2)}`,
            message,
            messageType,
            duration: 3000,
          })),
        ]);
      }
    },
    [removeGameId, scrollTo, setGameMessages, setAppMessages],
  );

  const [connectionId, sendWebSocketMessage, isWebsocketOpen] = useWebSocket(
    WEBSOCKET_ENDPOINT,
    onWebSocketMessage,
  );

  useEffect(() => {
    gameRecords.forEach((record) => {
      if (!gameIds.includes(record.game_id)) {
        addGameIdToURL(record.game_id);
      }
    });
  }, [gameRecords, gameIds, addGameIdToURL]);

  useEffect(() => {
    setUsername(usernameFromLocalStorage);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const [hasSentInitialJoinRequests, setHasSentInitialJoinRequests] =
    useState(false);

  useEffect(() => {
    if (
      !isWebsocketOpen ||
      !gameIdsFromUrl.length ||
      !username ||
      hasSentInitialJoinRequests
    ) {
      return;
    }

    gameIdsFromUrl.forEach((gameId) => {
      sendWebSocketMessage({
        route: API_ROUTE,
        data: {
          [PlayerActionName.JoinGame]: {
            username,
            gameId,
          },
        },
      });
    });

    setHasSentInitialJoinRequests(true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    gameIdsFromUrl,
    hasSentInitialJoinRequests,
    isWebsocketOpen,
    sendWebSocketMessage,
    // `username` left out to avoid sending a join request on first character,
    // when visiting 'join game' link without a username in local storage
  ]);

  const onLeaveGame = (gameId: string) => {
    setGameRecords((old) => old.filter((g) => g.game_id !== gameId));
    removeGameId(gameId);

    sendWebSocketMessage({
      route: API_ROUTE,
      data: {
        [PlayerActionName.LeaveGame]: {
          gameId,
        },
      },
    });
  };

  const onPlayAgain = (gameId: string) => {
    pendingPlayAgainIndexRef.current = gameRecords.findIndex(
      (g) => g.game_id === gameId,
    );
  };

  return (
    <div className="app-container">
      {gameIds.length ? (
        <MenuButtons
            realismOn={realismOn}
            setRealismPref={setRealismPref}
            pvOn={pvOn}
            setPvPref={setPvPref}
          />
      ) : null}

      {appMessages.length ? (
        <div className="app-messages-container">
          {appMessages.map((message) => (
            <Alert
              key={message.id}
              message={message}
              onDismiss={() => {
                dismissAppMessage(message.id);
              }}
            />
          ))}
        </div>
      ) : null}

      <HeaderSection setFormToShow={setFormToShow} setShowForm={setShowForm} />

      {showForm && (
        <>
          <div
            className="options-bar"
            onClick={() => {
              setShowForm((old) => !old);
            }}
          >
            Hide form
          </div>
          <div className="form-container">
            <GameForm
              sendWebSocketMessage={sendWebSocketMessage}
              mode={formToShow}
              setShowForm={setShowForm}
              setUsername={setUsername}
              gameIds={gameIds}
            />
          </div>
        </>
      )}

      {connectionId && (
        <div className="games-container">
          {gameRecords.map((gameRecord) => (
            <Game
              key={gameRecord.game_id}
              gameRecord={gameRecord}
              onLeaveGame={onLeaveGame}
              onPlayAgain={onPlayAgain}
              connectionId={connectionId}
              messages={gameMessages.filter((message) =>
                message.id.includes(gameRecord.game_id),
              )}
              sendWebSocketMessage={sendWebSocketMessage}
              dismissMessage={dismissGameMessage}
              totalActiveGames={gameRecords.length}
              aiAnalysis={aiAnalyses[gameRecord.game_id] ?? null}
              onRequestAiAnalysis={(gameId) => {
                pendingAiGameIdRef.current = gameId;
              }}
              onClearAiAnalysis={(gameId) => {
                setAiAnalyses((old) =>
                  Object.fromEntries(
                    Object.entries(old).filter(([id]) => id !== gameId),
                  ),
                );
              }}
              pvOn={pvOn}
            />
          ))}
        </div>
      )}
    </div>
  );
};
