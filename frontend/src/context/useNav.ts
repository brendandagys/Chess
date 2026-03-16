import { createContext, useContext } from "react";

interface INavContext {
  gameIds: string[];
  addGameId: (id: string) => void;
  removeGameId: (id: string) => void;
  username: string;
  setUsername: (username: string) => void;
}

export const NavContext = createContext({} as INavContext);

export const useNav = () => useContext(NavContext);
