import { formatTime } from "../utils";

import "../css/PlayerTime.css";

interface PlayerTimeProps {
  secondsLeft: number;
}

export const PlayerTime: React.FC<PlayerTimeProps> = ({ secondsLeft }) => {
  return (
    <div
      className={`player-time${secondsLeft < 60 ? " player-time--low" : ""}`}
    >
      <span>{formatTime(secondsLeft)}</span>
    </div>
  );
};
