import { Board, BoardSetup, Position } from "./board";
import { Color } from "./piece";

interface GameEndingCheckmate { 'checkmate': Color; }
interface GameEndingResignation { 'resignation': Color; }
interface GameEndingOutOfTime { 'out-of-time': Color; }
type GameEndingStalemate = 'stalemate';
type GameEndingDrawByThreefoldRepetition = 'draw-by-threefold-repetition';
type GameEndingDrawByFiftyMoveRule = 'draw-by-fifty-move-rule';
type GameEndingDrawByInsufficientMaterial = 'draw-by-insufficient-material';
type GameEndingDrawByMutualAgreement = 'draw-by-mutual-agreement';

type GameEnding =
  | GameEndingCheckmate
  | GameEndingResignation
  | GameEndingOutOfTime
  | GameEndingStalemate
  | GameEndingDrawByThreefoldRepetition
  | GameEndingDrawByFiftyMoveRule
  | GameEndingDrawByInsufficientMaterial
  | GameEndingDrawByMutualAgreement;

type StateNotStarted = 'not-started';
type StateInProgress = 'in-progress';
interface StateFinished {
  'finished': GameEnding;
}

type State =
  | StateNotStarted
  | StateInProgress
  | StateFinished;

interface GameState {
  gameId: string;
  state: State;
  currentTurn: Color;
  inCheck: Color | null;
  board: Board;
  move_history: unknown[];
}

export interface PlayerMove {
  from: Position;
  to: Position;
}

interface PlayerActionCreateGame {
  'create-game': {
    username: string;
    gameId: string | null;
    boardSetup: BoardSetup | null;
    colorPreference: Color | null;
  };
}

interface PlayerActionJoinGame {
  'join-game': {
    username: string;
    gameId: string;
  };
}

interface PlayerActionGetGameState {
  'get-game-state': {
    gameId: string;
  };
}

interface PlayerActionMovePiece {
  'move-piece': {
    gameId: string;
    playerMove: PlayerMove;
  };
}

interface PlayerActionResign {
  'resign': {
    gameId: string;
  };
}

interface PlayerActionOfferDraw {
  'offer-draw': {
    gameId: string;
  };
}

export type PlayerAction =
  | PlayerActionCreateGame
  | PlayerActionJoinGame
  | PlayerActionGetGameState
  | PlayerActionMovePiece
  | PlayerActionResign
  | PlayerActionOfferDraw;
