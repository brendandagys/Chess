import { Piece, PieceType, Color } from "@src/types/piece";
import { CompactBoard, ExpandedBoard } from "@src/types/board";
import { CompactCapturedPieces, ExpandedCapturedPieces } from "./types/game";

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
  };
};

// Decode base64-encoded board into a 2D array of Piece | null.
// The encoding is: 1 byte per square, 0-11 for pieces, 255 for empty.
// The back-end must also provide dimensions and move history for each piece.
export const getSquaresFromCompactBoard = (
  compactBoard: CompactBoard): ExpandedBoard["squares"] => {
  const { squares: base64, dimensions } = compactBoard;

  const bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
  const numSquares = dimensions.ranks * dimensions.files;

  const board: (Piece | null)[][] =
    Array.from({ length: dimensions.ranks }, () => []);

  for (let index = 0; index < numSquares; index++) {
    const byte = bytes[Math.floor(index / 2)];
    const nibble = (index % 2 === 0) ? (byte >> 4) & 0xF : byte & 0xF;
    const piece = nibble === 0xF ? null : decodePiece(nibble);
    const rank = Math.floor(index / dimensions.files);
    const file = index % dimensions.files;
    board[dimensions.ranks - 1 - rank][file] = piece;
  }

  return board;
};


export const getCapturedPiecesFromBase64 = (
  compactCapturedPieces: CompactCapturedPieces): ExpandedCapturedPieces => {
  const {
    [Color.White]: whiteBase64,
    [Color.Black]: blackBase64,
    whitePoints,
    blackPoints
  } = compactCapturedPieces;

  const whiteBytes = Uint8Array.from(atob(whiteBase64), (c) => c.charCodeAt(0));
  const blackBytes = Uint8Array.from(atob(blackBase64), (c) => c.charCodeAt(0));

  const whitePieces: Piece[] = Array.from(whiteBytes)
    .map((byte) => decodePiece(byte))
    .filter((p) => p !== null);

  const blackPieces: Piece[] = Array.from(blackBytes)
    .map((byte) => decodePiece(byte))
    .filter((p) => p !== null);

  return {
    [Color.White]: whitePieces,
    [Color.Black]: blackPieces,
    whitePoints,
    blackPoints,
  };
};
