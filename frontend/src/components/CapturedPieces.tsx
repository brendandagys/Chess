import { imageMap } from "@src/images";
import { Piece } from "@src/types/piece";

import "@src/css/CapturedPieces.css";

interface CapturedPiecesProps {
  pieces: Piece[];
  pointsLead: number;
}

export const CapturedPieces: React.FC<CapturedPiecesProps> = ({
  pieces,
  pointsLead,
}) => (
  <div className="captured-pieces">
    {pieces.map((piece, i) => (
      <div
        key={`${piece.color}-${piece.pieceType}-${i}`}
        className="image-container"
      >
        <img
          src={imageMap[piece.pieceType][piece.color]}
          alt={`${piece.color} ${piece.pieceType}`}
        />
      </div>
    ))}

    {pointsLead > 0 && <div className="score">+{pointsLead}</div>}
  </div>
);
