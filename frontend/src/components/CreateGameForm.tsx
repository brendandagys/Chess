import { useState } from "react";
import { API_ROUTE } from "../constants";
import { PlayerActionName } from "../types/game";
import { GameRequest } from "../types/api";

import { BoardSetup, BoardSetupName } from "../types/board";
import { Color } from "../types/piece";
import { getRandomIntInRange } from "../utils";

import "../css/CreateGameForm.css";

interface CreateGameFormProps {
  sendMessage: (action: GameRequest) => void;
}

export const CreateGameForm: React.FC<CreateGameFormProps> = ({
  sendMessage,
}) => {
  const [username, setUsername] = useState("");
  const [gameId, setGameId] = useState("");
  const [boardSetupName, setBoardSetupName] = useState<BoardSetupName>(
    BoardSetupName.Standard
  );
  const [colorPreference, setColorPreference] = useState<Color>(Color.White);

  const getBoardSetup = (name: BoardSetupName): BoardSetup => {
    switch (name) {
      case BoardSetupName.Standard:
        return BoardSetupName.Standard;
      case BoardSetupName.Random:
        return {
          [BoardSetupName.Random]: {
            ranks: getRandomIntInRange(6, 12),
            files: getRandomIntInRange(6, 12),
          },
        };
      case BoardSetupName.KingAndOneOtherPiece:
        return {
          [BoardSetupName.KingAndOneOtherPiece]: {
            ranks: 8,
            files: 8,
          },
        };
      default:
        return BoardSetupName.Standard;
    }
  };

  const handleCreateGame = (e: React.FormEvent) => {
    e.preventDefault();

    sendMessage({
      route: API_ROUTE,
      data: {
        [PlayerActionName.CreateGame]: {
          username,
          gameId: gameId || null,
          boardSetup: getBoardSetup(boardSetupName),
          colorPreference,
        },
      },
    });
  };

  return (
    <form className="create-game-form" onSubmit={handleCreateGame}>
      <div className="game-preferences-container">
        <div className="game-preferences-form-component">
          <span className="toggle-label">Play {colorPreference}</span>

          <label className="toggle">
            <input
              type="checkbox"
              defaultChecked={true}
              onChange={() => {
                setColorPreference((old) =>
                  old === Color.White ? Color.Black : Color.White
                );
              }}
            />
            <span className="slider"></span>
          </label>
        </div>

        <div className="game-preferences-form-component">
          <span className="board-setup-label">Board setup</span>

          <select
            className="board-setup-select"
            value={boardSetupName}
            onChange={(e) => {
              setBoardSetupName(e.target.value as BoardSetupName);
            }}
          >
            <option value="standard">Standard</option>
            <option value="random">Random</option>
            <option value="king-and-one-other-piece">
              King and 1 Other Piece
            </option>
          </select>
        </div>
      </div>

      <div className="game-details-container">
        <input
          className="username-field"
          placeholder="Username"
          value={username}
          onChange={(e) => {
            setUsername(e.target.value);
          }}
        />

        <input
          className="game-id-field"
          placeholder="Game ID (optional)"
          value={gameId}
          onChange={(e) => {
            setGameId(e.target.value);
          }}
        />

        <button type="submit">Create Game</button>
      </div>
    </form>
  );
};
