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

type BoardSetupStandard = 'standard';

interface BoardSetupRandom {
  'random': BoardSetupDimensions;
}

interface BoardSetupKingAndOneOtherPiece {
  'king-and-one-other-piece': BoardSetupDimensions;
}

export type BoardSetup =
  | BoardSetupStandard
  | BoardSetupRandom
  | BoardSetupKingAndOneOtherPiece;

export interface Board {
  squares: (Piece | null)[][];
}
