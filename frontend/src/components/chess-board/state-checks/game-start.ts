import { ExpandedGameStateAtPointInTime } from "@src/types/board";
import { playGameStartSound } from "../../../sounds";
import { GameStateType } from "../../../types/game";

const didStateChange = (
  one: ExpandedGameStateAtPointInTime,
  two: ExpandedGameStateAtPointInTime,
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
