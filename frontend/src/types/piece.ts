export enum Color {
  White = 'white',
  Black = 'black',
}

enum PieceType {
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
