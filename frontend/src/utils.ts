import { Piece, PieceType, Color } from "@src/types/piece";
import { CompactBoard, ExpandedBoard } from "@src/types/board";

// Returns a new matrix, equivalent to rotating the original matrix 180 degrees
export const rotateMatrix180Degrees = <T>(matrix: T[][]): T[][] => (
  matrix.map((row) => [...row].reverse()).reverse()
);

export const capitalizeFirstLetter = (str: string): string =>
  str.charAt(0).toUpperCase() + str.slice(1);

export const getLast = <T>(arr: T[]): T => {
  if (arr.length === 0) {
    throw new Error("Array is empty");
  }
  return arr[arr.length - 1];
};

export const formatTime = (seconds: number): string => {
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;

  return `${minutes}:${remainingSeconds < 10 ? "0" : ""}${remainingSeconds}`;
};


export const decodePiece = (code: number): Piece | null => {
  if (code === 255) return null;

  const color = code >= 6 ? Color.Black : Color.White;

  const typeIndex = code % 6;

  const pieceType = [
    PieceType.Pawn,
    PieceType.Knight,
    PieceType.Bishop,
    PieceType.Rook,
    PieceType.Queen,
    PieceType.King,
  ][typeIndex];

  return {
    pieceType,
    color,
    lastGameMove: null, // Last game move will be set later
  };
};

// Decode base64-encoded board into a 2D array of Piece | null.
// The encoding is: 1 byte per square, 0-11 for pieces, 255 for empty.
// The back-end must also provide dimensions and move history for each piece.
export const getBoardFromBase64 = (
  compactBoard: CompactBoard): ExpandedBoard["squares"] => {
  const { squares: base64, dimensions, lastGameMoves } = compactBoard;

  const bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));

  const board: (Piece | null)[][] =
    Array.from({ length: dimensions.ranks }, () => []);

  for (let rank = 0; rank < dimensions.ranks; rank++) {
    const row: (Piece | null)[] = [];

    for (let file = 0; file < dimensions.files; file++) {
      const index = rank * dimensions.files + file;

      const piece = decodePiece(bytes[index]);

      if (piece) {
        piece.lastGameMove = lastGameMoves[index];
      }

      row.push(piece);
    }

    board[dimensions.ranks - 1 - rank] = row;
  }

  return board;
};
