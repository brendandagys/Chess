/* eslint-disable max-len */
export const WEBSOCKET_ENDPOINT = "wss://3gd4hwwxc0.execute-api.us-east-1.amazonaws.com/Prod";
export const API_ROUTE = "game";

export interface BoardTheme {
  id: string;
  label: string;
  darkColor: string;
  lightColor: string;
  borderColor: string;
  blackPieceFilter?: string;
}

export const BOARD_THEMES: BoardTheme[] = [
  { id: "classic", label: "Classic", darkColor: "#779556", lightColor: "#ebecd0", borderColor: "#713c0b" },
  { id: "blue", label: "Blue", darkColor: "#7599ae", lightColor: "#d9e4e8", borderColor: "#366f90" },
  { id: "walnut", label: "Walnut", darkColor: "#b88762", lightColor: "#f0d9b5", borderColor: "#713c0b" },
  { id: "purple", label: "Purple", darkColor: "#886cb3", lightColor: "#dfd2e8", borderColor: "#703e75" },
  { id: "midnight", label: "Midnight", darkColor: "#1a1a2e", lightColor: "#61618c", borderColor: "#0d0d1a", blackPieceFilter: "drop-shadow(0 0 3px rgba(255, 255, 255, 0.9))" },
  // { id: "maroon", label: "Maroon", darkColor: "#a64d3d", lightColor: "#f5dbc3", borderColor: "#393636" },
];

// Scores above this threshold represent forced checkmate, not a material
// advantage. Matches the MATE_THRESHOLD constant used in the chess engine.
export const MATE_SCORE_THRESHOLD = 9000;
