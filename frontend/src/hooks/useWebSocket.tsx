import { useCallback, useEffect, useRef, useState } from "react";
import { ApiResponse, GameRequest } from "../types/api";

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
      websocket.current?.close();
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

  return [connectionId, sendMessage, isWebsocketOpen] as const;
};
