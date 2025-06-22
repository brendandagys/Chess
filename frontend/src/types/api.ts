import { PlayerAction } from "@src/types/game";

export interface GameRequest {
  route: string;
  data: PlayerAction;
}

export enum ApiMessageType {
  Info = 'info',
  Warning = 'warning',
  Error = 'error',
  Success = 'success',
}

interface ApiMessage {
  message: string;
  messageType: ApiMessageType;
}

export interface ApiResponse<T> {
  statusCode: number;
  connectionId: string | null;
  messages: ApiMessage[];
  data: T;
}
