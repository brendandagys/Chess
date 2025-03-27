/* eslint-disable @typescript-eslint/no-base-to-string */
import { GameRecord } from "../types/game";
import { Color } from "../types/piece";
import { ChessBoard } from "./ChessBoard";

import "../css/Game.css";

interface GameProps {
  gameRecord: GameRecord;
  usernames: string[];
}

export const Game: React.FC<GameProps> = ({ gameRecord, usernames }) => {
  const gameState = gameRecord.game_state;

  const playerColor = usernames.includes(gameRecord.white_username ?? "")
    ? Color.White
    : Color.Black;

  const isTurn = playerColor === gameState.currentTurn;

  return (
    <div className="game-container">
      <h2>Game: {gameRecord.game_id}</h2>
      <div className="status-container">
        <p className="pill pill--green">
          Game {gameState.state.toString().replace("-", " ")}
        </p>
        <p className="pill pill--blue">
          {gameState.currentTurn.toString().replace("w", "W").replace("b", "B")}
          {"'s turn"}
        </p>
        <p className="pill pill--pink">Playing as {playerColor}</p>
      </div>

      <div className={`chess-board-container ${isTurn && "is-player-turn"}`}>
        <ChessBoard board={gameState.board} playerColor={playerColor} />
      </div>
    </div>
  );
};
