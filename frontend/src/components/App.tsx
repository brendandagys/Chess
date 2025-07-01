import { useState, useCallback, useEffect } from "react";

import { useNav } from "@src/context/useNav";
import { useLocalStorage } from "@src/hooks/useLocalStorage";
import { useMessageDisplay } from "@src/hooks/useMessageDisplay";
import { useScroll } from "@src/hooks/useScroll";
import { useWebSocket } from "@src/hooks/useWebSocket";

import { Alert } from "@src/components/Alert";
import { CopyLinkButton } from "@src/components/CopyLinkButton";
import { HeaderSection } from "@src/components/HeaderSection";
import { GameForm } from "@src/components/GameForm";
import { Game } from "@src/components/Game";

import { ApiMessageType, ApiResponse } from "@src/types/api";
import { GameRecord, PlayerActionName } from "@src/types/game";
import { FormToShow } from "@src/types/sharedComponentTypes";
import { API_ROUTE, WEBSOCKET_ENDPOINT } from "@src/constants";

import { playIllegalSound } from "@src/sounds";
import "@src/css/App.css";

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

  const [appMessages, setAppMessages, dismissAppMessage] = useMessageDisplay();
  const [gameMessages, setGameMessages, dismissGameMessage] =
    useMessageDisplay();

  const [formToShow, setFormToShow] = useState<FormToShow>(
    !username && gameIds.length ? FormToShow.Join : FormToShow.Create
  );
  const [showForm, setShowForm] = useState(
    !/^\/game\/(.+)$/.exec(window.location.pathname) ||
      !usernameFromLocalStorage
  );

  const scrollTo = useScroll();

  const onWebSocketMessage = useCallback(
    (response: ApiResponse<GameRecord | null>) => {
      const isGameRecord =
        response.data && Object.keys(response.data).includes("game_id");

      const gameRecord = isGameRecord ? response.data : null;

      if (gameRecord) {
        setGameRecords((old) => {
          const index = old.findIndex(
            (game) => game.game_id === gameRecord.game_id
          );

          if (index === -1) {
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
                  m.message === o.message && m.messageType === o.messageType
              )
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
    [scrollTo, setGameMessages, setAppMessages]
  );

  const [connectionId, sendWebSocketMessage, isWebsocketOpen] = useWebSocket(
    WEBSOCKET_ENDPOINT,
    onWebSocketMessage
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

  return (
    <div className="app-container">
      {gameIds.length ? <CopyLinkButton /> : null}

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
              connectionId={connectionId}
              messages={gameMessages.filter((message) =>
                message.id.includes(gameRecord.game_id)
              )}
              sendWebSocketMessage={sendWebSocketMessage}
              dismissMessage={dismissGameMessage}
            />
          ))}
        </div>
      )}
    </div>
  );
};
