import { useCallback } from "react";

export const useScroll = () => {
  const scrollTo = useCallback((targetId: string, offset = 0) => {
    const targetElement = document.getElementById(targetId);

    if (targetElement) {
      const yPosition =
        targetElement.getBoundingClientRect().top + window.scrollY + offset;

      window.scrollTo({ top: yPosition, behavior: "smooth" });
    }
  }, []);

  return scrollTo;
};
