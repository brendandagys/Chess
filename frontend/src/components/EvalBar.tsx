import { Color } from "@src/types/piece";
import { MATE_SCORE_THRESHOLD } from "@src/constants";

import "@src/css/EvalBar.css";

interface EvalBarProps {
  evaluation: number;
  isBookMove: boolean;
  playerColor: Color;
}

function evalToHumanPercent(evalCp: number): number {
  const humanEvalCp = -evalCp;

  if (Math.abs(humanEvalCp) > MATE_SCORE_THRESHOLD) {
    return humanEvalCp > 0 ? 97 : 3;
  }

  // Linear (clamped): less responsive near center, more extreme at edges
  const clamped = Math.max(-1000, Math.min(1000, humanEvalCp));
  return 50 + (clamped / 1000) * 50;
}

function formatEval(evalCp: number): string {
  const humanEvalCp = -evalCp;

  if (Math.abs(humanEvalCp) > MATE_SCORE_THRESHOLD) {
    return humanEvalCp > 0 ? "M" : "−M";
  }

  const pawns = humanEvalCp / 100;

  return pawns >= 0 ? `+${pawns.toFixed(1)}` : pawns.toFixed(1);
}

export const EvalBar: React.FC<EvalBarProps> = ({
  evaluation,
  playerColor,
}) => {
  const isPlayerWhite = playerColor === Color.White;

  const humanPercent = evalToHumanPercent(evaluation);
  const enginePercent = 100 - humanPercent;

  const topColor = isPlayerWhite ? "black" : "white";
  const bottomColor = isPlayerWhite ? "white" : "black";

  const labelText = formatEval(evaluation);

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
