// Returns a new matrix, equivalent to rotating the original matrix 180 degrees
export const rotateMatrix180Degrees = <T>(matrix: T[][]): T[][] => (
  matrix.map((row) => [...row].reverse()).reverse()
);
