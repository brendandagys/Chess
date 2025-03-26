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

export interface GameState {
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

export enum PlayerActionName {
  CreateGame = 'create-game',
  JoinGame = 'join-game',
  GetGameState = 'get-game-state',
  MovePiece = 'move-piece',
  Resign = 'resign',
  OfferDraw = 'offer-draw',
}

interface PlayerActionCreateGame {
  [PlayerActionName.CreateGame]: {
    username: string;
    gameId: string | null;
    boardSetup: BoardSetup | null;
    colorPreference: Color | null;
  };
}


interface PlayerActionJoinGame {
  [PlayerActionName.JoinGame]: {
    username: string;
    gameId: string;
  };
}

interface PlayerActionGetGameState {
  [PlayerActionName.GetGameState]: {
    gameId: string;
  };
}

interface PlayerActionMovePiece {
  [PlayerActionName.MovePiece]: {
    gameId: string;
    playerMove: PlayerMove;
  };
}

interface PlayerActionResign {
  [PlayerActionName.Resign]: {
    gameId: string;
  };
}

interface PlayerActionOfferDraw {
  [PlayerActionName.OfferDraw]: {
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

export interface GameRecord {
  game_id: string;
  white_connection_id: string | null;
  white_username: string | null;
  black_connection_id: string | null;
  black_username: string | null;
  game_state: GameState;
  created: string;
}
