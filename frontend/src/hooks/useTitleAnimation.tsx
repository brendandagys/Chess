import { useEffect, useRef } from "react";

export const useTitleAnimation = (
  shouldAnimate: boolean,
  message: string,
  intervalMs = 1000
) => {
  const originalTitle = useRef(document.title);
  const intervalId = useRef<number | null>(null);

  useEffect(() => {
    const title = originalTitle.current;

    if (shouldAnimate) {
      let showMessage = true;

      intervalId.current = window.setInterval(() => {
        document.title = showMessage ? message : title;
        showMessage = !showMessage;
      }, intervalMs);

      return () => {
        if (intervalId.current) {
          clearInterval(intervalId.current);
          intervalId.current = null;
        }
        document.title = title;
      };
    } else {
      document.title = title;
    }
  }, [shouldAnimate, message, intervalMs]);
};
