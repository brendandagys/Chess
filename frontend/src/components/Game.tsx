/* eslint-disable @typescript-eslint/no-base-to-string */
import { useState, useEffect } from "react";
import { GameRecord } from "../types/game";
import { Color } from "../types/piece";
import { ChessBoard } from "./ChessBoard";
import { Alert } from "./Alert";
import { GameMessage, GameMessageColor } from "../types/sharedComponentTypes";

import "../css/Game.css";

interface GameProps {
  gameRecord: GameRecord;
  usernames: string[];
}

export const Game: React.FC<GameProps> = ({ gameRecord, usernames }) => {
  const [gameMessages, setGameMessages] = useState<GameMessage[]>([]);

  const gameState = gameRecord.game_state;

  const playerColor = usernames.includes(gameRecord.white_username ?? "")
    ? Color.White
    : Color.Black;

  const isTurn = playerColor === gameState.currentTurn;

  useEffect(() => {
    const timers = gameMessages.map((message) =>
      setTimeout(() => {
        setGameMessages((old) => old.filter((m) => m.id !== message.id));
      }, message.duration)
    );
    return () => {
      timers.forEach(clearTimeout);
    };
  }, [gameMessages]);

  const dismissAlert = (id: string) => {
    setGameMessages((old) => old.filter((message) => message.id !== id));
  };

  return (
    <div className="game-container">
      <h2 className="game-id">Game: {gameRecord.game_id}</h2>

      {gameMessages.length ? (
        <div className="messages-container">
          {gameMessages.map((message) => (
            <Alert
              key={message.id}
              message={message}
              onDismiss={() => {
                dismissAlert(message.id);
              }}
            />
          ))}
        </div>
      ) : (
        <div className="status-container">
          <p className="pill pill--green">
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
