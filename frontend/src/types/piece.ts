export enum Color {
  White = 'white',
  Black = 'black',
}

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
  hasMoved: boolean;
}
