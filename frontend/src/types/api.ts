import { PlayerAction } from "./game";

export interface GameRequest {
  route: string;
  data: PlayerAction;
}

export enum ApiErrorType {
  Info = 'info',
  Warning = 'warning',
  Error = 'error',
  Success = 'success',
}

interface ApiErrorMessage {
  message: string;
  errorType: ApiErrorType;
}

export interface ApiResponse<T> {
  statusCode: number;
  connectionId: string | null;
  messages: ApiErrorMessage[];
  data: T | null;
}
