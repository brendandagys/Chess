import { useState } from "react";
import { useLocalStorage } from "../hooks/useLocalStorage";
import { API_ROUTE } from "../constants";
import { ColorPreference, PlayerActionName, TimeOption } from "../types/game";
import { GameRequest } from "../types/api";
import { FormToShow } from "../types/sharedComponentTypes";
import { BoardSetup, BoardSetupName } from "../types/board";

import "../css/GameForm.css";

interface GameFormProps {
  sendWebSocketMessage: (action: GameRequest) => void;
  mode: FormToShow;
  setShowForm: React.Dispatch<React.SetStateAction<boolean>>;
  setUsername: (username: string) => void;
  gameIds: string[];
  hiddenGameIds: string[];
}

export const GameForm: React.FC<GameFormProps> = ({
  sendWebSocketMessage,
  mode,
  setShowForm,
  setUsername,
  gameIds,
  hiddenGameIds,
}) => {
  const [username, setUsernameInLocalStorage] = useLocalStorage("username", "");

  const [gameId, setGameId] = useState(
    gameIds.length === 1 && !username ? gameIds[0] : ""
  );

  const [timeOption, setTimeOption] = useState<TimeOption>(
    TimeOption.Unlimited
  );

  const [boardSetupName, setBoardSetupName] = useState<BoardSetupName>(
    BoardSetupName.Standard
  );

  const [dimensions, setDimensions] = useState({
    ranks: "8",
    files: "8",
  });

  const [colorPreference, setColorPreference] = useState<ColorPreference>(
    ColorPreference.Random
  );

  const getBoardSetup = (name: BoardSetupName): BoardSetup => {
    switch (name) {
      case BoardSetupName.Standard:
        return BoardSetupName.Standard;
      case BoardSetupName.Random:
        return {
          [BoardSetupName.Random]: {
            ranks: parseInt(dimensions.ranks) || 8,
            files: parseInt(dimensions.files) || 8,
          },
        };
      case BoardSetupName.KingAndOneOtherPiece:
        return {
          [BoardSetupName.KingAndOneOtherPiece]: {
            ranks: parseInt(dimensions.ranks) || 8,
            files: parseInt(dimensions.files) || 8,
          },
        };
      default:
        return BoardSetupName.Standard;
    }
  };

  const onSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    const joinPayload = hiddenGameIds.includes(gameId)
      ? {
          [PlayerActionName.GetGameState]: {
            gameId,
          },
        }
      : {
          [PlayerActionName.JoinGame]: {
            username,
            gameId,
          },
        };

    const data =
      mode === FormToShow.Create
        ? {
            [PlayerActionName.CreateGame]: {
              username,
              gameId: gameId || null,
              boardSetup: getBoardSetup(boardSetupName),
              colorPreference,
              secondsPerPlayer:
                timeOption === TimeOption.Unlimited ? null : timeOption,
            },
          }
        : joinPayload;

    sendWebSocketMessage({
      route: API_ROUTE,
      data,
    });

    setShowForm(false);
    setGameId("");
  };

  return (
    <form className="game-form" onSubmit={onSubmit}>
      {mode === FormToShow.Create && (
        <div className="game-preferences-container">
          <div className="game-preferences-form-component">
            <span className="label">Color</span>

            <select
              className="color-preference-select"
              value={colorPreference}
              onChange={(e) => {
                setColorPreference(e.target.value as ColorPreference);
              }}
            >
              <option value={ColorPreference.Random}>Random</option>
              <option value={ColorPreference.White}>White</option>
              <option value={ColorPreference.Black}>Black</option>
            </select>
          </div>

          <div className="game-preferences-form-component">
            <span className="label">Time</span>

            <select
              className="board-setup-select"
              value={timeOption}
              onChange={(e) => {
                setTimeOption(parseInt(e.target.value, 10) as TimeOption);
              }}
            >
              <option value={TimeOption.OneMinute}>1 Minute</option>
              <option value={TimeOption.ThreeMinutes}>3 Minutes</option>
              <option value={TimeOption.FiveMinutes}>5 Minutes</option>
              <option value={TimeOption.TenMinutes}>10 Minutes</option>
              <option value={TimeOption.FifteenMinutes}>15 Minutes</option>
              <option value={TimeOption.ThirtyMinutes}>30 Minutes</option>
              <option value={TimeOption.OneHour}>1 Hour</option>
              <option value={TimeOption.Unlimited}>Unlimited</option>
            </select>
          </div>

          <div className="game-preferences-form-component">
            <span className="label">Board setup</span>

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

          {boardSetupName !== BoardSetupName.Standard && (
            <div style={{ display: "flex", gap: "0.8rem" }}>
              <div className="game-preferences-form-component">
                <span className="label">
                  Ranks <small>(6-12)</small>
                </span>
                <input
                  type="number"
                  min="6"
                  max="12"
                  value={dimensions.ranks}
                  onChange={(e) => {
                    if (/^(1|6|7|8|9|10|11|12)?$/.exec(e.target.value)) {
                      setDimensions((old) => ({
                        ...old,
                        ranks: e.target.value.trim(),
                      }));
                    }
                  }}
                />
              </div>

              <div className="game-preferences-form-component">
                <span className="label">
                  Files <small>(6-12)</small>
                </span>
                <input
                  type="number"
                  min="6"
                  max="12"
                  value={dimensions.files}
                  onChange={(e) => {
                    if (/^(1|6|7|8|9|10|11|12)?$/.exec(e.target.value)) {
                      setDimensions((old) => ({
                        ...old,
                        files: e.target.value.trim(),
                      }));
                    }
                  }}
                />
              </div>
            </div>
          )}
        </div>
      )}

      <div className="game-details-container">
        <input
          type="text"
          className="username-field"
          placeholder="Username"
          value={username}
          onChange={(e) => {
            const username = e.target.value.trim();
            setUsernameInLocalStorage(username);
            setUsername(username);
          }}
        />

        <input
          type="text"
          className="game-id-field"
          placeholder={`Game ID${
            mode === FormToShow.Create ? " (optional)" : ""
          }`}
          value={gameId}
          onChange={(e) => {
            setGameId(e.target.value);
          }}
        />

        <button
          type="submit"
          disabled={
            !username.trim() || (mode === FormToShow.Join && !gameId.trim())
          }
          className={`main-action-button main-action-button${
            mode === FormToShow.Create ? "--secondary" : ""
          }`}
        >{`${mode === FormToShow.Create ? "Start" : "Join"} game`}</button>
      </div>
    </form>
  );
};
