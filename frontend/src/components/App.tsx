import { useState, useCallback, useRef, useEffect } from "react";

import { useNav } from "../context/useNav";
import { useLocalStorage } from "../hooks/useLocalStorage";
import { useMessageDisplay } from "../hooks/useMessageDisplay";
import { useWebSocket } from "../hooks/useWebSocket";

import { GameForm } from "./GameForm";
import { Alert } from "./Alert";
import { Game } from "./Game";
import { CopyLinkButton } from "./CopyLinkButton";

import { ApiResponse } from "../types/api";
import { GameRecord, PlayerActionName } from "../types/game";
import { FormToShow } from "../types/sharedComponentTypes";
import { API_ROUTE, WEBSOCKET_ENDPOINT } from "../constants";

import "../css/App.css";
import _moveSound from "../sounds/move-self.mp3";
import hero from "../images/hero.png";

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

  const moveSound = useRef<HTMLAudioElement>(new Audio(_moveSound));

  const onWebSocketMessage = useCallback(
    (response: ApiResponse<unknown>) => {
      const isGameRecord = Object.keys(response.data ?? {}).includes("game_id");
      const gameRecord = isGameRecord ? (response.data as GameRecord) : null;

      if (gameRecord) {
        setGameRecords((old) => {
          const index = old.findIndex(
            (game) => game.game_id === gameRecord.game_id
          );

          if (index === -1) {
            return [...old, gameRecord];
          }

          // Play move-piece sound if board has changed since last update
          if (
            !old[index].game_state.board.squares.every((row, r) =>
              row.every((oldPiece, c) => {
                const newBoardPiece = gameRecord.game_state.board.squares[r][c];

                return (
                  newBoardPiece?.color === oldPiece?.color &&
                  newBoardPiece?.pieceType === oldPiece?.pieceType
                );
              })
            )
          ) {
            void moveSound.current.play();
          }

          const newGames = [...old];
          newGames[index] = gameRecord;

          document.querySelectorAll(".dragging").forEach((el) => {
            el.classList.remove("dragging");
          });

          return newGames;
        });
      }

      if (response.messages.length) {
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
    [setGameMessages, setAppMessages]
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
  }, [
    gameIdsFromUrl,
    hasSentInitialJoinRequests,
    isWebsocketOpen,
    sendWebSocketMessage,
    username,
  ]);

  const [hiddenGameIds, setHiddenGameIds] = useState<string[]>([]);

  const onHideGame = (gameId: string) => {
    setGameRecords((old) => old.filter((g) => g.game_id !== gameId));
    setHiddenGameIds((old) => [...old, gameId]);
    removeGameId(gameId);
  };

  return (
    <div className="app-container">
      {gameIds.length ? <CopyLinkButton /> : null}

      <div className="title-container">
        {appMessages.length ? (
          appMessages.map((message) => (
            <Alert
              key={message.id}
              message={message}
              onDismiss={() => {
                dismissAppMessage(message.id);
              }}
            />
          ))
        ) : (
          <img
            src={hero}
            alt="Play Chess"
            className="hero-image"
            onClick={() => {
              window.location.href = "/";
            }}
          />
        )}
      </div>

      <p className="sub-title">
        <button
          onClick={() => {
            setFormToShow(FormToShow.Create);
            setShowForm(true);
          }}
          className="button-link"
        >
          Start a new game
        </button>
        {" or "}
        <button
          onClick={() => {
            setFormToShow(FormToShow.Join);
            setShowForm(true);
          }}
          className="button-link"
        >
          Join an existing game
        </button>
      </p>

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
              hiddenGameIds={hiddenGameIds}
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
              onHideGame={onHideGame}
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
