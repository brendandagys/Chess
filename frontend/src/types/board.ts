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
  CustomSize = 'custom-size',
  Chess960 = 'chess960',
  KingAndKnights = 'king-and-knights',
}

export type BoardSetupStandard = 'standard';

interface BoardSetupCustomSize {
  [BoardSetupName.CustomSize]: BoardDimensions;
}

type BoardSetupChess960 = 'chess960';
interface BoardSetupKingAndKnights {
  [BoardSetupName.KingAndKnights]: BoardDimensions;
}

export type BoardSetup =
  | BoardSetupStandard
  | BoardSetupCustomSize
  | BoardSetupChess960
  | BoardSetupKingAndKnights;

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
