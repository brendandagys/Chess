import { useEffect, useRef } from "react";

export const useNotifications = () => {
  const permissionRequested = useRef(false);

  useEffect(() => {
    if (
      "Notification" in window &&
      Notification.permission === "default" &&
      !permissionRequested.current
    ) {
      permissionRequested.current = true;
    }
  }, []);

  const requestPermission = async (): Promise<boolean> => {
    if (!("Notification" in window)) {
      console.warn("This browser does not support notifications");
      return false;
    }

    if (Notification.permission === "granted") {
      return true;
    }

    if (Notification.permission === "denied") {
      return false;
    }

    const permission = await Notification.requestPermission();
    return permission === "granted";
  };

  const showNotification = (
    title: string,
    options?: NotificationOptions
  ): Notification | null => {
    if (!("Notification" in window) || Notification.permission !== "granted") {
      return null;
    }

    if (!document.hidden) {
      return null;
    }

    try {
      return new Notification(title, {
        icon: "/chess-logo.svg",
        badge: "/chess-logo.svg",
        ...options,
      });
    } catch (error) {
      console.error("Error showing notification:", error);
      return null;
    }
  };

  return { requestPermission, showNotification };
};
