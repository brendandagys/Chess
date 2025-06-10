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
