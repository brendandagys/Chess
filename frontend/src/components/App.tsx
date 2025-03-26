import { useState, useCallback } from "react";
import { useWebSocket } from "../hooks/useWebSocket";
import { CreateGameForm } from "./CreateGameForm";
import { ChessBoard } from "./ChessBoard";
import { GameRecord } from "../types/game";
import { WEBSOCKET_ENDPOINT } from "../constants";

import "../css/App.css";

export const App: React.FC = () => {
  const [gameRecord, setGameRecord] = useState<GameRecord | null>(null);

  const onMessage = useCallback((gameRecord: GameRecord) => {
    setGameRecord(gameRecord);
  }, []);

  const sendMessage = useWebSocket(WEBSOCKET_ENDPOINT, onMessage);

  return (
    <div>
      <h1 className="title">Play Chess</h1>
      <p className="sub-title">Start a new game or join an existing one</p>

      <div className="form-container">
        <CreateGameForm sendMessage={sendMessage} />
      </div>

      {gameRecord && <ChessBoard gameRecord={gameRecord} />}
    </div>
  );
};
