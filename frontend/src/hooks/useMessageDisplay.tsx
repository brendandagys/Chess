import { useState, useEffect } from "react";
import { GameMessage } from "../types/sharedComponentTypes";

export const useMessageDisplay = () => {
  const [messages, setMessages] = useState<GameMessage[]>([]);

  useEffect(() => {
    const timers = messages.map((message) =>
      setTimeout(() => {
        setMessages((old) => old.filter((m) => m.id !== message.id));
      }, message.duration)
    );

    return () => {
      timers.forEach(clearTimeout);
    };
  }, [messages]);

  const dismissMessage = (id: string) => {
    setMessages((old) => old.filter((message) => message.id !== id));
  };

  return [messages, setMessages, dismissMessage] as const;
};
