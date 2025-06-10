import { useCallback, useEffect, useRef, useState } from "react";

import { ApiResponse, GameRequest } from "@src/types/api";
import { PlayerActionName } from "@src/types/game";
import { API_ROUTE } from "@src/constants";

export const useWebSocket = (
  url: string,
  onMessage: (response: ApiResponse<unknown>) => void
) => {
  const websocket = useRef<WebSocket | null>(null);
  const [isWebsocketOpen, setIsWebsocketOpen] = useState(false);
  const [connectionId, setConnectionId] = useState<string | null>(null);

  useEffect(() => {
    websocket.current = new WebSocket(url);

    websocket.current.onopen = () => {
      setIsWebsocketOpen(true);
      console.log("WebSocket connected");
    };

    websocket.current.onmessage = (event: MessageEvent) => {
      const response = JSON.parse(event.data as string) as ApiResponse<unknown>;
      console.info("Received message:", response);
      setConnectionId(response.connectionId);
      onMessage(response);
    };

    websocket.current.onerror = (error) => {
      console.error("WebSocket error", error);
    };

    websocket.current.onclose = () => {
      setIsWebsocketOpen(false);
      console.info("WebSocket disconnected");
    };

    return () => {
      if (websocket.current) {
        websocket.current.onopen = null;
        websocket.current.onmessage = null;
        websocket.current.onerror = null;
        websocket.current.onclose = null;
        websocket.current.close();
        websocket.current = null;
      }
    };
  }, [url, onMessage]);

  const sendMessage = useCallback((message: GameRequest) => {
    if (websocket.current?.readyState === WebSocket.OPEN) {
      console.info("Sending message:", message);
      websocket.current.send(JSON.stringify(message));
    } else {
      console.error("WebSocket is not open");
    }
  }, []);

  // Keep the connection alive
  useEffect(() => {
    const intervalId = setInterval(() => {
      sendMessage({ route: API_ROUTE, data: PlayerActionName.Heartbeat });
    }, 30 * 1000);

    return () => {
      clearInterval(intervalId);
    };
  }, [sendMessage]);

  return [connectionId, sendMessage, isWebsocketOpen] as const;
};
