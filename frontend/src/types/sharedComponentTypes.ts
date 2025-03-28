import { ApiErrorType } from "./api";

export enum FormToShow {
  Create = "create",
  Join = "join",
}

export interface GameMessage {
  id: string;
  message: string;
  errorType: ApiErrorType;
  duration: number;
}
