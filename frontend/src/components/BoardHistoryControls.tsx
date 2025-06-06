import "../css/BoardHistoryControls.css";

interface BoardHistoryControlsProps {
  historyIndex: number;
  setHistoryIndex: React.Dispatch<React.SetStateAction<number>>;
  numStates: number;
}

export const BoardHistoryControls: React.FC<BoardHistoryControlsProps> = ({
  historyIndex,
  setHistoryIndex,
  numStates,
}) => {
  return (
    <div className="board-history-controls">
      <button
        disabled={historyIndex === 0}
        onClick={() => {
          setHistoryIndex((prev) => Math.max(0, prev - 1));
        }}
      >
        &lt; Previous
      </button>
      <span>
        State {historyIndex + 1} of {numStates}
      </span>
      <button
        disabled={historyIndex === numStates - 1}
        onClick={() => {
          setHistoryIndex((prev) => Math.min(numStates - 1, prev + 1));
        }}
      >
        Next &gt;
      </button>
    </div>
  );
};
