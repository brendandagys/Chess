import { ApiMessageType } from "./api";

export enum FormToShow {
  Create = "create",
  Join = "join",
}

export interface GameMessage {
  id: string;
  message: string;
  messageType: ApiMessageType;
  duration: number;
}
