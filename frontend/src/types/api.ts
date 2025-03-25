import { PlayerAction } from "./game";

export interface GameRequest {
  route: string;
  data: PlayerAction;
}

export interface ApiResponse<T> {
  statusCode: number;
  message: string | null;
  data: T | null;
}
