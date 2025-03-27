import { Piece } from "./piece";

// 0-indexed
type Rank = number;
type File = number;

export interface Position {
  rank: Rank;
  file: File;
}

interface BoardSetupDimensions {
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
  [BoardSetupName.Random]: BoardSetupDimensions;
}

interface BoardSetupKingAndOneOtherPiece {
  [BoardSetupName.KingAndOneOtherPiece]: BoardSetupDimensions;
}

export type BoardSetup =
  | BoardSetupStandard
  | BoardSetupRandom
  | BoardSetupKingAndOneOtherPiece;

export interface Board {
  squares: (Piece | null)[][];
}
