import { useState, useCallback, useRef } from "react";
import { useWebSocket } from "../hooks/useWebSocket";
import { GameForm } from "./GameForm";
import { FormToShow } from "../types/sharedComponentTypes";
import { GameRecord } from "../types/game";
import { Game } from "./Game";
import { WEBSOCKET_ENDPOINT } from "../constants";
import { useMessageDisplay } from "../hooks/useMessageDisplay";
import { Alert } from "./Alert";
import { ApiResponse } from "../types/api";

import _moveSound from "../sounds/move-self.mp3";
import "../css/App.css";

export const App: React.FC = () => {
  const [appMessages, setAppMessages, dismissAppMessage] = useMessageDisplay();
  const [gameMessages, setGameMessages, dismissGameMessage] =
    useMessageDisplay();
  const [gameRecords, setGameRecords] = useState<GameRecord[]>([]);
  const [showForm, setShowForm] = useState(true);
  const [formToShow, setFormToShow] = useState<FormToShow>(FormToShow.Create);

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

          return newGames;
        });
      }

      if (response.messages.length) {
        (gameRecord ? setGameMessages : setAppMessages)((old) => [
          ...old,
          ...response.messages.map(({ message, messageType }) => ({
            id: `${
              gameRecord?.game_id ? `${gameRecord.game_id}-` : ""
            }${Math.random().toString(36).slice(2)}`,
            message,
            messageType,
            duration: 5000,
          })),
        ]);
      }
    },
    [setAppMessages, setGameMessages]
  );

  const [connectionId, sendWebSocketMessage] = useWebSocket(
    WEBSOCKET_ENDPOINT,
    onWebSocketMessage
  );

  return (
    <div className="app-container">
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
          <h1 className="main-title">Play Chess</h1>
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
