import { GameStateAtPointInTime } from '../../../types/game';
import { Color } from '../../../types/piece';

import { moveSelf } from './move-self';
import { moveOpponent } from './move-opponent';
import { moveCheck } from './check';
import { capture } from './capture';
import { gameStart } from './game-start';
import { promote } from './promote';
import { castle } from './castle';

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
