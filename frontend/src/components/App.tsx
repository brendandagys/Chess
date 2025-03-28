import { useState, useCallback } from "react";
import { useWebSocket } from "../hooks/useWebSocket";
import { GameForm } from "./GameForm";
import { FormToShow } from "../types/sharedComponentTypes";
import { GameRecord } from "../types/game";
import { Game } from "./Game";
import { WEBSOCKET_ENDPOINT } from "../constants";
import { useMessageDisplay } from "../hooks/useMessageDisplay";
import { Alert } from "./Alert";

import "../css/App.css";
import { ApiResponse } from "../types/api";

export const App: React.FC = () => {
  const [messages, setMessages, dismissMessage] = useMessageDisplay();
  const [gameRecords, setGameRecords] = useState<GameRecord[]>([]);
  const [showForm, setShowForm] = useState(true);
  const [formToShow, setFormToShow] = useState<FormToShow>(FormToShow.Create);
  const [usernames, setUsernames] = useState<string[]>([]);

  const onWebSocketMessage = useCallback(
    (response: ApiResponse<unknown>) => {
      if (response.messages.length) {
        setMessages((old) => [
          ...old,
          ...response.messages.map(({ message, errorType }) => ({
            id: crypto.randomUUID(),
            message,
            errorType,
            duration: 5000,
          })),
        ]);
      }

      if (response.data as GameRecord | null) {
        const gameRecord = response.data as GameRecord;

        setGameRecords((old) => [
          ...old.filter((game) => game.game_id !== gameRecord.game_id),
          gameRecord,
        ]);
      }
    },
    [setMessages]
  );

  const sendMessage = useWebSocket(WEBSOCKET_ENDPOINT, onWebSocketMessage);

  return (
    <div className="app-container">
      <div className="title-container">
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
          <h1>Play Chess</h1>
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
        <div
          className="options-bar"
          onClick={() => {
            setShowForm((old) => !old);
          }}
        >
          Hide form
        </div>
      )}

      {showForm && (
        <div className="form-container">
          <GameForm
            sendMessage={sendMessage}
            mode={formToShow}
            setUsernames={setUsernames}
          />
        </div>
      )}

      <div className="games-container">
        {gameRecords.map((gameRecord) => (
          <Game
            key={gameRecord.game_id}
            gameRecord={gameRecord}
            usernames={usernames}
          />
        ))}
      </div>
    </div>
  );
};
