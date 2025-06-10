import { GameStateAtPointInTime } from "@src/types/game";
import { Color } from "@src/types/piece";

import { moveSelf } from "@src/components/chess-board/state-checks/move-self";
import {
  moveOpponent
} from "@src/components/chess-board/state-checks/move-opponent";
import { moveCheck } from "@src/components/chess-board/state-checks/check";
import { capture } from "@src/components/chess-board/state-checks/capture";
import { gameStart } from "@src/components/chess-board/state-checks/game-start";
import { promote } from "@src/components/chess-board/state-checks/promote";
import { castle } from "@src/components/chess-board/state-checks/castle";

interface StateCheck {
  didStateChange: (
    old: GameStateAtPointInTime,
    cur: GameStateAtPointInTime,
    playerColor: Color,
  ) => boolean;
  action: () => void;
  name: string;
}

export const stateChecks: StateCheck[] = [
  gameStart,
  moveCheck,
  promote,
  castle,
  capture,
  moveSelf,
  moveOpponent,
];
