export const getRandomIntInRange = (
  minimum: number, maximum: number
): number => {
  if (minimum > maximum) {
    throw new Error("Min should not be greater than Max");
  }

  const variance = maximum - minimum + 1;

  return minimum + Math.floor(Math.random() * variance);
};


export const rotateBoard180Degrees = <T>(matrix: T[][]): T[][] => (
  matrix.map((row) => [...row].reverse()).reverse()
);
