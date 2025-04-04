import { Board, BoardSetup, Position } from "./board";
import { Color, Piece } from "./piece";

export enum GameEndingType {
  Checkmate = 'checkmate',
  Resignation = 'resignation',
  OutOfTime = 'out-of-time',
  Stalemate = 'stalemate',
  DrawByThreefoldRepetition = 'draw-by-threefold-repetition',
  DrawByFiftyMoveRule = 'draw-by-fifty-move-rule',
  DrawByInsufficientMaterial = 'draw-by-insufficient-material',
  DrawByMutualAgreement = 'draw-by-mutual-agreement',
}

export interface GameEndingCheckmate { [GameEndingType.Checkmate]: Color; }
interface GameEndingResignation { [GameEndingType.Resignation]: Color; }
interface GameEndingOutOfTime { [GameEndingType.OutOfTime]: Color; }
type GameEndingStalemate = GameEndingType.Stalemate;
type GameEndingDrawByThreefoldRepetition =
  GameEndingType.DrawByThreefoldRepetition;
type GameEndingDrawByFiftyMoveRule = GameEndingType.DrawByFiftyMoveRule;
type GameEndingDrawByInsufficientMaterial =
  GameEndingType.DrawByInsufficientMaterial;
type GameEndingDrawByMutualAgreement = GameEndingType.DrawByMutualAgreement;

type GameEnding =
  | GameEndingCheckmate
  | GameEndingResignation
  | GameEndingOutOfTime
  | GameEndingStalemate
  | GameEndingDrawByThreefoldRepetition
  | GameEndingDrawByFiftyMoveRule
  | GameEndingDrawByInsufficientMaterial
  | GameEndingDrawByMutualAgreement;

export enum GameStateType {
  NotStarted = 'not-started',
  InProgress = 'in-progress',
  Finished = 'finished',
}

type StateNotStarted = GameStateType.NotStarted;
type StateInProgress = GameStateType.InProgress;
interface StateFinished {
  [GameStateType.Finished]: GameEnding;
}

type State =
  | StateNotStarted
  | StateInProgress
  | StateFinished;

export interface CapturedPieces {
  [Color.White]: Piece[];
  [Color.Black]: Piece[];
  whitePoints: number;
  blackPoints: number;
}

export interface GameState {
  gameId: string;
  state: State;
  currentTurn: Color;
  inCheck: Color | null;
  board: Board;
  move_history: unknown[];
  capturedPieces: CapturedPieces;
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
