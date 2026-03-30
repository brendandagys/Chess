import { Color } from "@src/types/piece";
import { MATE_SCORE_THRESHOLD } from "@src/constants";

import "@src/css/EvalBar.css";

interface EvalBarProps {
  normalizedEvaluation: number;
  isBookMove: boolean;
  playerColor: Color;
}

function evalToHumanPercent(evalCp: number): number {
  if (Math.abs(evalCp) > MATE_SCORE_THRESHOLD) {
    return evalCp > 0 ? 97 : 3;
  }

  // Linear (clamped): less responsive near center, more extreme at edges
  const clamped = Math.max(-1000, Math.min(1000, evalCp));
  return 50 + (clamped / 1000) * 50;
}

function formatEval(evalCp: number): string {
  if (Math.abs(evalCp) > MATE_SCORE_THRESHOLD) {
    return evalCp > 0 ? "M" : "−M";
  }

  const pawns = evalCp / 100;

  return pawns >= 0 ? `+${pawns.toFixed(1)}` : pawns.toFixed(1);
}

export const EvalBar: React.FC<EvalBarProps> = ({
  normalizedEvaluation, // From human's perspective (negated in Game.tsx)
  playerColor,
}) => {
  const isPlayerWhite = playerColor === Color.White;

  const humanPercent = evalToHumanPercent(normalizedEvaluation);
  const enginePercent = 100 - humanPercent;

  const topColor = isPlayerWhite ? "black" : "white";
  const bottomColor = isPlayerWhite ? "white" : "black";

  const labelText = formatEval(normalizedEvaluation);

  return (
    <div className="eval-bar">
      <div
        className={`eval-bar__${topColor}`}
        style={{ flex: `${enginePercent} 0 0%` }}
      />
      <div
        className={`eval-bar__${bottomColor}`}
        style={{ flex: `${humanPercent} 0 0%` }}
      />
      {labelText && (
        <span className="eval-bar__label" style={{ top: `${enginePercent}%` }}>
          {labelText}
        </span>
      )}
    </div>
  );
};
