import { useState, useCallback } from "react";
import { useWebSocket } from "../hooks/useWebSocket";
import { GameForm } from "./GameForm";
import { FormToShow } from "../types/sharedComponentTypes";
import { ChessBoard } from "./ChessBoard";
import { GameRecord } from "../types/game";
import { WEBSOCKET_ENDPOINT } from "../constants";

import "../css/App.css";

export const App: React.FC = () => {
  const [gameRecords, setGameRecords] = useState<GameRecord[]>([]);
  const [showForm, setShowForm] = useState(true);
  const [formToShow, setFormToShow] = useState<FormToShow>(FormToShow.Create);

  const onMessage = useCallback((gameRecord: GameRecord) => {
    setGameRecords((old) => [
      ...old.filter((game) => game.game_id !== gameRecord.game_id),
      gameRecord,
    ]);
  }, []);

  const sendMessage = useWebSocket(WEBSOCKET_ENDPOINT, onMessage);

  return (
    <div>
      <h1 className="title">Play Chess</h1>
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

      <div
        className="options-bar"
        onClick={() => {
          setShowForm((old) => !old);
        }}
      >
        Hide form
      </div>

      {showForm && (
        <div className="form-container">
          <GameForm sendMessage={sendMessage} mode={formToShow} />
        </div>
      )}

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
