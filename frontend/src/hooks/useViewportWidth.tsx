import { useState, useEffect } from "react";

interface ViewportInformation {
  width: number;
}

export const useViewportWidth = (): ViewportInformation => {
  const [width, setWidth] = useState(window.innerWidth);

  useEffect(() => {
    const onResize = () => {
      setWidth(window.innerWidth);
    };

    window.addEventListener("resize", onResize);

    return () => {
      window.removeEventListener("resize", onResize);
    };
  }, []);

  return { width };
};
