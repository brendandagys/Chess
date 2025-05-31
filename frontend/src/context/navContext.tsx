import { ReactNode, useEffect, useState } from "react";
import { NavContext } from "./useNav";

// Helper to parse game IDs from the path
function parseGameIdsFromPath(pathname: string): string[] {
  const match = /^\/game\/(.+)$/.exec(pathname);
  const ids = match?.[1]?.split(",").filter(Boolean) ?? [];
  return Array.from(new Set(ids));
}

// Helper to build path from game IDs and username
function buildPathFromGameIds(gameIds: string[], username?: string): string {
  let path = gameIds.length ? `/game/${gameIds.join(",")}` : "/";
  if (username) {
    path += `?username=${encodeURIComponent(username)}`;
  }
  return path;
}

// Helper to parse username from query string
function parseUsernameFromSearch(search: string): string {
  const params = new URLSearchParams(search);
  return params.get("username")?.trim() ?? "";
}

export const NavProvider = ({ children }: { children: ReactNode }) => {
  const [gameIds, setGameIdsState] = useState<string[]>(() =>
    parseGameIdsFromPath(window.location.pathname)
  );

  const [username, setUsernameState] = useState<string>(() =>
    parseUsernameFromSearch(window.location.search)
  );

  useEffect(() => {
    const handler = (e: PopStateEvent) => {
      const state = e.state as { gameIds: string[]; username: string };

      const gameIds =
        e.state && Array.isArray(state.gameIds)
          ? state.gameIds
          : parseGameIdsFromPath(window.location.pathname);

      const user =
        e.state && typeof state.username === "string"
          ? state.username
          : parseUsernameFromSearch(window.location.search);

      setGameIdsState(gameIds);
      setUsernameState(user);
    };

    window.addEventListener("popstate", handler);

    return () => {
      window.removeEventListener("popstate", handler);
    };
  }, []);

  // Update URL and state
  const setGameIds = (ids: string[]) => {
    setGameIdsState([...ids]);
    const path = buildPathFromGameIds(ids, username);
    history.pushState({ gameIds: ids, username }, "", path);
  };

  const setUsername = (user: string) => {
    setUsernameState(user);
    const path = buildPathFromGameIds(gameIds, user);
    history.pushState({ gameIds, username: user }, "", path);
  };

  const addGameId = (id: string) => {
    setGameIdsState((prev) => {
      if (prev.includes(id)) return prev;
      const next = [...prev, id];
      const path = buildPathFromGameIds(next, username);
      history.pushState({ gameIds: next, username }, "", path);
      return next;
    });
  };

  const removeGameId = (id: string) => {
    setGameIdsState((prev) => {
      const next = prev.filter((gameId) => gameId !== id);
      const path = buildPathFromGameIds(next, username);
      history.pushState({ gameIds: next, username }, "", path);
      return next;
    });
  };

  return (
    <NavContext.Provider
      value={{
        gameIds,
        username,
        setGameIds,
        addGameId,
        removeGameId,
        setUsername,
      }}
    >
      {children}
    </NavContext.Provider>
  );
};
