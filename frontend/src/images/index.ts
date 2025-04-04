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

export const images = {
  wp,
  bp,
  wr,
  br,
  wn,
  bn,
  wb,
  bb,
  wq,
  bq,
  wk,
  bk,
};

export const imageMap = {
  [PieceType.Pawn]: { white: images.wp, black: images.bp },
  [PieceType.Rook]: { white: images.wr, black: images.br },
  [PieceType.Knight]: { white: images.wn, black: images.bn },
  [PieceType.Bishop]: { white: images.wb, black: images.bb },
  [PieceType.Queen]: { white: images.wq, black: images.bq },
  [PieceType.King]: { white: images.wk, black: images.bk },
};
