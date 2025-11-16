import { useCallback, useEffect, useRef, useState } from "react";

import { ApiResponse, GameRequest } from "@src/types/api";
import { GameRecord, PlayerActionName } from "@src/types/game";
import { API_ROUTE } from "@src/constants";

const HEARTBEAT_INTERVAL_MS = 30_000;

const MAX_RECONNECT_ATTEMPTS = 5;
const INITIAL_RECONNECT_DELAY_MS = 1_000;
const MAX_RECONNECT_DELAY_MS = 30_000;

export const useWebSocket = (
  url: string,
  onMessage: (response: ApiResponse<GameRecord | null>) => void
) => {
  const websocket = useRef<WebSocket | null>(null);
  const [isWebsocketOpen, setIsWebsocketOpen] = useState(false);
  const [connectionId, setConnectionId] = useState<string | null>(null);
  const reconnectAttempts = useRef(0);
  const reconnectTimeout = useRef<number | null>(null);
  const isIntentionallyClosed = useRef(false);
  const heartbeatInterval = useRef<number | null>(null);

  const connect = useCallback(() => {
    if (
      websocket.current?.readyState === WebSocket.OPEN ||
      websocket.current?.readyState === WebSocket.CONNECTING
    ) {
      return;
    }

    console.info("Connecting to WebSocket...");
    websocket.current = new WebSocket(url);

    websocket.current.onopen = () => {
      setIsWebsocketOpen(true);
      reconnectAttempts.current = 0;
      console.info("WebSocket connected");
    };

    websocket.current.onmessage = (event: MessageEvent) => {
      const response = JSON.parse(
        event.data as string
      ) as ApiResponse<GameRecord | null>;
      console.debug("Received message:", response);
      setConnectionId(response.connectionId);
      onMessage(response);
    };

    websocket.current.onerror = (error) => {
      console.error("WebSocket error", error);
    };

    websocket.current.onclose = (event) => {
      setIsWebsocketOpen(false);

      console.info("WebSocket disconnected", {
        code: event.code,
        reason: event.reason,
        wasClean: event.wasClean,
      });

      // Only attempt reconnection if not intentionally closed
      if (!isIntentionallyClosed.current) {
        if (reconnectAttempts.current < MAX_RECONNECT_ATTEMPTS) {
          const delay = Math.min(
            INITIAL_RECONNECT_DELAY_MS * Math.pow(2, reconnectAttempts.current),
            MAX_RECONNECT_DELAY_MS
          );

          reconnectAttempts.current++;

          console.info(
            `Attempting to reconnect ` +
              `(${reconnectAttempts.current}/${MAX_RECONNECT_ATTEMPTS}) ` +
              `in ${delay}ms...`
          );

          reconnectTimeout.current = window.setTimeout(() => {
            connect();
          }, delay);
        } else {
          console.error(
            "Max reconnection attempts reached. Please refresh the page."
          );
        }
      }
    };
  }, [url, onMessage]);

  useEffect(() => {
    isIntentionallyClosed.current = false;
    connect();

    return () => {
      isIntentionallyClosed.current = true;

      if (reconnectTimeout.current) {
        clearTimeout(reconnectTimeout.current);
        reconnectTimeout.current = null;
      }

      if (heartbeatInterval.current) {
        clearInterval(heartbeatInterval.current);
        heartbeatInterval.current = null;
      }

      if (websocket.current) {
        websocket.current.onopen = null;
        websocket.current.onmessage = null;
        websocket.current.onerror = null;
        websocket.current.onclose = null;
        websocket.current.close();
        websocket.current = null;
      }
    };
  }, [connect]);

  const sendMessage = useCallback((message: GameRequest) => {
    if (websocket.current?.readyState === WebSocket.OPEN) {
      console.debug("Sending message:", message);
      websocket.current.send(JSON.stringify(message));
    } else {
      console.error("WebSocket is not open");
    }
  }, []);

  // Keep the connection alive
  useEffect(() => {
    if (isWebsocketOpen) {
      heartbeatInterval.current = window.setInterval(() => {
        sendMessage({ route: API_ROUTE, data: PlayerActionName.Heartbeat });
      }, HEARTBEAT_INTERVAL_MS);
    } else {
      if (heartbeatInterval.current) {
        clearInterval(heartbeatInterval.current);
        heartbeatInterval.current = null;
      }
    }

    return () => {
      if (heartbeatInterval.current) {
        clearInterval(heartbeatInterval.current);
        heartbeatInterval.current = null;
      }
    };
  }, [sendMessage, isWebsocketOpen]);

  return [connectionId, sendMessage, isWebsocketOpen] as const;
};
