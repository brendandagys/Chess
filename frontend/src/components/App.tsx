import { useState, useCallback } from "react";
import { useWebSocket } from "../hooks/useWebSocket";
import { CreateGameForm } from "./CreateGameForm";
import { ChessBoard } from "./ChessBoard";
import { GameRecord } from "../types/game";
import { WEBSOCKET_ENDPOINT } from "../constants";

import "../css/App.css";

export const App: React.FC = () => {
  const [gameRecords, setGameRecords] = useState<GameRecord[]>([]);

  const onMessage = useCallback((gameRecord: GameRecord) => {
    setGameRecords((old) => [...old, gameRecord]);
  }, []);

  const sendMessage = useWebSocket(WEBSOCKET_ENDPOINT, onMessage);

  return (
    <div>
      <h1 className="title">Play Chess</h1>
      <p className="sub-title">Start a new game or join an existing one</p>

      <div className="form-container">
        <CreateGameForm sendMessage={sendMessage} />
      </div>

      <div className="boards-container">
        {gameRecords.map((gameRecord) => (
          <div key={gameRecord.game_id} className="board-container">
            <ChessBoard gameRecord={gameRecord} />
          </div>
        ))}
      </div>
    </div>
  );
};
