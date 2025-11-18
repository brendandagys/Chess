import { BoardSetup, CompactBoard, Position } from "@src/types/board";
import { Color, Piece } from "@src/types/piece";

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
export interface GameEndingResignation { [GameEndingType.Resignation]: Color; }
export interface GameEndingOutOfTime { [GameEndingType.OutOfTime]: Color; }
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

export interface CompactCapturedPieces {
  [Color.White]: string;
  [Color.Black]: string;
  whitePoints: number;
  blackPoints: number;
}

export interface ExpandedCapturedPieces {
  [Color.White]: Piece[];
  [Color.Black]: Piece[];
  whitePoints: number;
  blackPoints: number;
}

export enum ColorPreference {
  White = 'white',
  Black = 'black',
  Random = 'random',
}

export enum EngineDifficulty {
  Beginner = 'beginner',
  Easy = 'easy',
  Medium = 'medium',
  Hard = 'hard',
  Expert = 'expert',
  Master = 'master',
}

export enum TimeOption {
  OneMinute = 60,
  ThreeMinutes = 180,
  FiveMinutes = 300,
  TenMinutes = 600,
  FifteenMinutes = 900,
  ThirtyMinutes = 1800,
  OneHour = 3600,
  Unlimited = -1,
}

interface GameTime {
  bothPlayersLastConnectedAt: string | null;
  lastMoveAt: string | null;
  whiteSecondsLeft: number;
  blackSecondsLeft: number;
}

export interface SearchStatistics {
  depth: number;
  nodes: number;
  qnodes: number;
  timeMs: number;
  fromBook: boolean;
}

export interface GameStateAtPointInTime {
  state: State;
  currentTurn: Color;
  inCheck: Color | null;
  board: CompactBoard;
  capturedPieces: CompactCapturedPieces;
  engineResult: SearchStatistics | null;
}

export interface GameState {
  gameId: string;
  gameTime: GameTime | null;
  history: GameStateAtPointInTime[];
}

export interface PlayerMove {
  from: Position;
  to: Position;
}

export enum PlayerActionName {
  CreateGame = 'create-game',
  JoinGame = 'join-game',
  LeaveGame = 'leave-game',
  GetGameState = 'get-game-state',
  MovePiece = 'move-piece',
  Heartbeat = 'heartbeat',
  LoseViaOutOfTime = 'lose-via-out-of-time',
  Resign = 'resign',
  OfferDraw = 'offer-draw',
}

interface PlayerActionCreateGame {
  [PlayerActionName.CreateGame]: {
    username: string;
    gameId: string | null;
    boardSetup: BoardSetup | null;
    colorPreference: ColorPreference;
    secondsPerPlayer: TimeOption | null;
    engineDifficulty: EngineDifficulty | null;
  };
}


interface PlayerActionJoinGame {
  [PlayerActionName.JoinGame]: {
    username: string;
    gameId: string;
  };
}

interface PlayerActionLeaveGame {
  [PlayerActionName.LeaveGame]: {
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

type PlayerActionHeartbeat = PlayerActionName.Heartbeat;

interface PlayerActionLoseViaOutOfTime {
  [PlayerActionName.LoseViaOutOfTime]: {
    gameId: string;
  };
}
interface PlayerActionResign {
  [PlayerActionName.Resign]: {
    gameId: string;
  };
}

// interface PlayerActionOfferDraw {
//   [PlayerActionName.OfferDraw]: {
//     gameId: string;
//   };
// }

export type PlayerAction =
  | PlayerActionCreateGame
  | PlayerActionJoinGame
  | PlayerActionLeaveGame
  | PlayerActionGetGameState
  | PlayerActionMovePiece
  | PlayerActionHeartbeat
  | PlayerActionLoseViaOutOfTime
  | PlayerActionResign;
// | PlayerActionOfferDraw;

export interface GameRecord {
  game_id: string;
  white_connection_id: string | null;
  white_username: string | null;
  black_connection_id: string | null;
  black_username: string | null;
  engine_difficulty: EngineDifficulty | null;
  game_state: GameState;
  created: string;
}
