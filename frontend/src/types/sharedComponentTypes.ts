export enum FormToShow {
  Create = "create",
  Join = "join",
}

export enum GameMessageColor {
  Green = "green",
  Blue = "blue",
  Red = "red",
}

export interface GameMessage {
  id: string;
  message: string;
  duration: number;
  color: GameMessageColor;
}
