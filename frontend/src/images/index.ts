import wp from "../images/wp.png";
import bp from "../images/bp.png";
import wr from "../images/wr.png";
import br from "../images/br.png";
import wn from "../images/wn.png";
import bn from "../images/bn.png";
import wb from "../images/wb.png";
import bb from "../images/bb.png";
import wq from "../images/wq.png";
import bq from "../images/bq.png";
import wk from "../images/wk.png";
import bk from "../images/bk.png";

import { PieceType } from "../types/piece";

export const imageMap = {
  [PieceType.Pawn]: { white: wp, black: bp },
  [PieceType.Rook]: { white: wr, black: br },
  [PieceType.Knight]: { white: wn, black: bn },
  [PieceType.Bishop]: { white: wb, black: bb },
  [PieceType.Queen]: { white: wq, black: bq },
  [PieceType.King]: { white: wk, black: bk },
};
