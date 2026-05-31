import { useEffect, useRef, useState } from "react";

import Markdown from "react-markdown";

import { AiAnalysisResult } from "@src/types/game";

import "@src/css/AiAnalysisModal.css";

interface Props {
  isLoading: boolean;
  aiAnalysis: AiAnalysisResult | null;
  onClose: () => void;
  onRegenerate: () => void;
}

type DragMode = "move" | "resize-n" | "resize-se";

const MIN_WIDTH = 220;
const MIN_HEIGHT = 120;

export const AiAnalysisModal: React.FC<Props> = ({
  isLoading,
  aiAnalysis,
  onClose,
  onRegenerate,
}) => {
  const [layout, setLayout] = useState(() => ({
    x: Math.max(20, window.innerWidth - 420),
    y: 80,
    width: 360,
    height: 420,
  }));

  const dragMode = useRef<DragMode | null>(null);
  const dragStart = useRef({
    mouseX: 0,
    mouseY: 0,
    x: 0,
    y: 0,
    width: 0,
    height: 0,
  });

  const beginDrag = (mode: DragMode, clientX: number, clientY: number) => {
    dragMode.current = mode;

    dragStart.current = {
      mouseX: clientX,
      mouseY: clientY,
      x: layout.x,
      y: layout.y,
      width: layout.width,
      height: layout.height,
    };
  };

  useEffect(() => {
    const onMove = (clientX: number, clientY: number) => {
      if (!dragMode.current) return;

      const dx = clientX - dragStart.current.mouseX;
      const dy = clientY - dragStart.current.mouseY;

      if (dragMode.current === "move") {
        setLayout((prev) => ({
          ...prev,
          x: Math.max(0, dragStart.current.x + dx),
          y: Math.max(0, dragStart.current.y + dy),
        }));
      } else if (dragMode.current === "resize-n") {
        const newHeight = Math.max(MIN_HEIGHT, dragStart.current.height - dy);
        const newY =
          dragStart.current.y + (dragStart.current.height - newHeight);
        setLayout((prev) => ({ ...prev, y: newY, height: newHeight }));
      } else {
        setLayout((prev) => ({
          ...prev,
          width: Math.max(MIN_WIDTH, dragStart.current.width + dx),
          height: Math.max(MIN_HEIGHT, dragStart.current.height + dy),
        }));
      }
    };

    const handleMouseMove = (e: MouseEvent) => {
      onMove(e.clientX, e.clientY);
    };
    const handleMouseUp = () => {
      dragMode.current = null;
    };
    const handleTouchMove = (e: TouchEvent) => {
      const t = e.touches[0];
      onMove(t.clientX, t.clientY);
    };
    const handleTouchEnd = () => {
      dragMode.current = null;
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    window.addEventListener("touchmove", handleTouchMove);
    window.addEventListener("touchend", handleTouchEnd);

    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
      window.removeEventListener("touchmove", handleTouchMove);
      window.removeEventListener("touchend", handleTouchEnd);
    };
  }, []);

  return (
    <div
      className="ai-modal"
      style={{
        left: layout.x,
        top: layout.y,
        width: layout.width,
        height: layout.height,
      }}
    >
      <div
        className="ai-modal__resize-n"
        onMouseDown={(e) => {
          beginDrag("resize-n", e.clientX, e.clientY);
          e.preventDefault();
        }}
        onTouchStart={(e) => {
          beginDrag("resize-n", e.touches[0].clientX, e.touches[0].clientY);
          e.preventDefault();
        }}
      />

      <div
        className="ai-modal__header"
        onMouseDown={(e) => {
          beginDrag("move", e.clientX, e.clientY);
          e.preventDefault();
        }}
        onTouchStart={(e) => {
          beginDrag("move", e.touches[0].clientX, e.touches[0].clientY);
          e.preventDefault();
        }}
      >
        <span className="ai-modal__title">AI Analysis</span>
        <div className="ai-modal__header-actions">
          <button
            className="ai-modal__regenerate"
            onClick={onRegenerate}
            disabled={isLoading}
            aria-label="Regenerate analysis"
            title="Regenerate"
          >
            ↻
          </button>
          <button
            className="ai-modal__close"
            onClick={onClose}
            aria-label="Close"
          >
            ✕
          </button>
        </div>
      </div>

      <div className="ai-modal__body">
        {isLoading && !aiAnalysis && (
          <div className="ai-analysis-skeleton">
            <div
              className={
                "ai-analysis-skeleton__line ai-analysis-skeleton__line--wide"
              }
            />
            <div className="ai-analysis-skeleton__line" />
            <div
              className={
                "ai-analysis-skeleton__line ai-analysis-skeleton__line--medium"
              }
            />
            <div
              className={
                "ai-analysis-skeleton__line ai-analysis-skeleton__line--narrow"
              }
            />
          </div>
        )}
        {aiAnalysis && (
          <div className="ai-analysis-text">
            <Markdown>{aiAnalysis.text}</Markdown>
          </div>
        )}
      </div>

      <div
        className="ai-modal__resize-se"
        onMouseDown={(e) => {
          beginDrag("resize-se", e.clientX, e.clientY);
          e.preventDefault();
        }}
        onTouchStart={(e) => {
          beginDrag("resize-se", e.touches[0].clientX, e.touches[0].clientY);
          e.preventDefault();
        }}
      />
    </div>
  );
};
