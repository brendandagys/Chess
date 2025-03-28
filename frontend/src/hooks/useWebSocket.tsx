import { useEffect, useRef } from "react";
import { ApiResponse, GameRequest } from "../types/api";

export const useWebSocket = (
  url: string,
  onMessage: (response: ApiResponse<unknown>) => void
) => {
  const websocket = useRef<WebSocket | null>(null);

  useEffect(() => {
    websocket.current = new WebSocket(url);

    websocket.current.onopen = () => {
      console.log("WebSocket connected");
    };

    websocket.current.onmessage = (event: MessageEvent) => {
      const response = JSON.parse(event.data as string) as ApiResponse<unknown>;

      onMessage(response);
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
};
