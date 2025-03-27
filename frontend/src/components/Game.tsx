/* eslint-disable @typescript-eslint/no-base-to-string */
import { useState, useEffect } from "react";
import { GameRecord } from "../types/game";
import { Color } from "../types/piece";
import { ChessBoard } from "./ChessBoard";

import "../css/Game.css";

interface GameProps {
  gameRecord: GameRecord;
  usernames: string[];
}

export const Game: React.FC<GameProps> = ({ gameRecord, usernames }) => {
  const [alertMessage, setAlertMessage] = useState<string | null>(null);
  const [alertColor, setAlertColor] = useState<"green" | "blue" | "red">(
    "blue"
  );

  const gameState = gameRecord.game_state;

  const playerColor = usernames.includes(gameRecord.white_username ?? "")
    ? Color.White
    : Color.Black;

  const isTurn = playerColor === gameState.currentTurn;

  useEffect(() => {
    if (alertMessage) {
      const timer = setTimeout(() => {
        setAlertMessage(null);
      }, 5000);
      return () => {
        clearTimeout(timer);
      };
    }
  }, [alertMessage]);

  const dismissAlert = () => {
    setAlertMessage(null);
  };

  return (
    <div className="game-container">
      <h2 className="game-id">Game: {gameRecord.game_id}</h2>

      {alertMessage ? (
        <div
          className={`alert alert--${alertColor} ${
            !alertMessage ? "hidden" : ""
          }`}
        >
          <span className="message">{alertMessage}</span>
          <button className="alert-dismiss" onClick={dismissAlert}>
            &times;
          </button>
        </div>
      ) : (
        <div className="status-container">
          <p
            className="pill pill--green"
            onClick={() => {
              setAlertMessage("Game started!");
              setAlertColor("green");
            }}
          >
            Game {gameState.state.toString().replace("-", " ")}
          </p>
          <p className="pill pill--blue">
            {gameState.currentTurn
              .toString()
              .replace("w", "W")
              .replace("b", "B")}
            {"'s turn"}
          </p>
          <p className="pill pill--pink">Playing as {playerColor}</p>
        </div>
      )}

      <div className={`chess-board-container ${isTurn && "is-player-turn"}`}>
        <ChessBoard board={gameState.board} playerColor={playerColor} />
      </div>
    </div>
  );
};
