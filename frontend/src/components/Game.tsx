/* eslint-disable @typescript-eslint/no-base-to-string */
import { GameRecord } from "../types/game";
import { Color } from "../types/piece";
import { ChessBoard } from "./ChessBoard";
import { Alert } from "./Alert";
import { GameMessage } from "../types/sharedComponentTypes";

import "../css/Game.css";

interface GameProps {
  gameRecord: GameRecord;
  usernames: string[];
  messages: GameMessage[];
  dismissMessage: (id: string) => void;
}

export const Game: React.FC<GameProps> = ({
  gameRecord,
  usernames,
  messages,
  dismissMessage,
}) => {
  const gameState = gameRecord.game_state;

  const playerColor = usernames.includes(gameRecord.white_username ?? "")
    ? Color.White
    : Color.Black;

  const isTurn = playerColor === gameState.currentTurn;

  return (
    <div className="game-container">
      <h2 className="game-id">Game: {gameRecord.game_id}</h2>

      <div className="status-container">
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
          <>
            <p className="pill pill--green">
              Game {gameState.state.toString().replace("-", " ")}
            </p>
            <p className={`pill pill--blue ${!isTurn ? "pill--faded" : ""}`}>
              {isTurn
                ? "Your turn!"
                : `${playerColor === Color.White ? "Black" : "White"}'s turn`}
            </p>
            <p className="pill pill--pink">Playing as {playerColor}</p>
          </>
        )}
      </div>

      <div className={`chess-board-container ${isTurn && "is-player-turn"}`}>
        <ChessBoard board={gameState.board} playerColor={playerColor} />
      </div>
    </div>
  );
};
