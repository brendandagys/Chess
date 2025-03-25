import { PlayerAction } from "./game";

interface GameRequest {
  route: string;
  data: PlayerAction;
}

interface ApiResponse<T> {
  statusCode: number;
  message: string | null;
  data: T | null;
}
