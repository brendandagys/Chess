import { GameStateAtPointInTime } from '../../../types/game';
import { Color } from '../../../types/piece';

import { moveSelf } from './move-self';
import { moveOpponent } from './move-opponent';

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
  moveSelf,
  moveOpponent
];
