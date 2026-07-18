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

export interface ApiRunTimeError {
  message: string;
  connectionId: string | null;
  requestId: string | null;
}

export interface ApiResponse<T> {
  statusCode: number;
  connectionId: string | null;
  messages: ApiMessage[];
  data: T;
  replacesGameId?: string;
}

export function isApiRunTimeError(
  response: ApiResponse<unknown> | ApiRunTimeError,
): response is ApiRunTimeError {
  return !("statusCode" in response);
}
