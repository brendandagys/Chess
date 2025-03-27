import { useEffect, useRef } from "react";
import { GameRecord } from "../types/game";
import { GameRequest } from "../types/api";

export function useWebSocket(
  url: string,
  onMessage: (gameRecord: GameRecord) => void
) {
  const websocket = useRef<WebSocket | null>(null);

  useEffect(() => {
    websocket.current = new WebSocket(url);

    websocket.current.onopen = () => {
      console.log("WebSocket connected");
    };

    websocket.current.onmessage = (event) => {
      const response = JSON.parse(event.data) as GameRecord;

      if (response.game_id) {
        onMessage(response);
      }
    };

    websocket.current.onerror = (error) => {
      console.error("WebSocket error", error);
    };

    websocket.current.onclose = () => {
      console.log("WebSocket disconnected");
    };

    return () => {
      websocket.current?.close();
    };
  }, [url, onMessage]);

  const sendMessage = (message: GameRequest) => {
    if (websocket.current?.readyState === WebSocket.OPEN) {
      console.log("Sending message:", message);
      websocket.current.send(JSON.stringify(message));
    } else {
      console.error("WebSocket is not open");
    }
  };

  return sendMessage;
}
