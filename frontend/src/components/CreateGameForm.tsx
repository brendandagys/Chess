import { useState } from "react";
import { API_ROUTE } from "../constants";
import { PlayerActionName } from "../types/game";
import { GameRequest } from "../types/api";

import "../css/CreateGameForm.css";

interface CreateGameFormProps {
  sendMessage: (action: GameRequest) => void;
}

export const CreateGameForm: React.FC<CreateGameFormProps> = ({
  sendMessage,
}) => {
  const [username, setUsername] = useState("");
  const [gameId, setGameId] = useState("");

  const handleCreateGame = (e: React.FormEvent) => {
    e.preventDefault();

    sendMessage({
      route: API_ROUTE,
      data: {
        [PlayerActionName.CreateGame]: {
          username,
          gameId: gameId || null,
          boardSetup: null,
          colorPreference: null,
        },
      },
    });
  };

  return (
    <form className="create-game-form" onSubmit={handleCreateGame}>
      <label>Username:</label>
      <input
        value={username}
        onChange={(e) => {
          setUsername(e.target.value);
        }}
      />

      <label>Game ID:</label>
      <input
        placeholder="Optional"
        value={gameId}
        onChange={(e) => {
          setGameId(e.target.value);
        }}
      />

      <button type="submit">Create Game</button>
    </form>
  );
};
