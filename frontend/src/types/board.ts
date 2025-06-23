import { Piece } from "@src/types/piece";
import { GameStateAtPointInTime } from "./game";

// 1-indexed on front-end!
type Rank = number;
type File = number;

export interface Position {
  rank: Rank;
  file: File;
}

interface BoardDimensions {
  ranks: number;
  files: number;
}

export enum BoardSetupName {
  Standard = 'standard',
  Random = 'random',
  KingAndOneOtherPiece = 'king-and-one-other-piece',
}

export type BoardSetupStandard = 'standard';

interface BoardSetupRandom {
  [BoardSetupName.Random]: BoardDimensions;
}

interface BoardSetupKingAndOneOtherPiece {
  [BoardSetupName.KingAndOneOtherPiece]: BoardDimensions;
}

export type BoardSetup =
  | BoardSetupStandard
  | BoardSetupRandom
  | BoardSetupKingAndOneOtherPiece;

export interface CompactBoard {
  squares: string; // Base64-encoded string of pieces
  moveCount: number;
  dimensions: BoardDimensions;
  lastGameMoves: (number | null)[];
}

type Squares = (Piece | null)[][];

export interface ExpandedBoard {
  squares: Squares;
}

export type ExpandedGameStateAtPointInTime = Omit<
  GameStateAtPointInTime, "board"> & {
    board: ExpandedBoard;
  };
