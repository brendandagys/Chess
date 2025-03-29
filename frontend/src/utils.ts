export const rotateBoard180Degrees = <T>(matrix: T[][]): T[][] => (
  matrix.map((row) => [...row].reverse()).reverse()
);
