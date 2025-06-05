import { playGameStartSound } from "../../../sounds";
import { GameStateAtPointInTime, GameStateType } from "../../../types/game";

const didStateChange = (
  one: GameStateAtPointInTime,
  two: GameStateAtPointInTime,
) => {
  return (
    one.state === GameStateType.NotStarted
    && two.state === GameStateType.InProgress
  );
};

const action = () => {
  playGameStartSound();
};

export const gameStart = {
  didStateChange,
  action,
  name: "game-start",
};
