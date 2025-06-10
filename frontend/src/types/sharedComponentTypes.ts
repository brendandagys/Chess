import { ApiMessageType } from "@src/types/api";

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
