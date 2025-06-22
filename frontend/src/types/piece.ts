export enum Color {
  White = 'white',
  Black = 'black',
}

export const getOppositePlayerColor = (playerColor: Color) =>
  playerColor === Color.White ? Color.Black : Color.White;

export enum PieceType {
  King = 'king',
  Queen = 'queen',
  Rook = 'rook',
  Bishop = 'bishop',
  Knight = 'knight',
  Pawn = 'pawn',
}

export interface Piece {
  pieceType: PieceType;
  color: Color;
  lastGameMove: number | null;
}
