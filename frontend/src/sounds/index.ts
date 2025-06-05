import moveSelfSound from "../sounds/move-self.mp3";
import moveOpponentSound from "../sounds/move-opponent.mp3";
import moveCheckSound from "../sounds/move-check.mp3";
import captureSound from "../sounds/capture.mp3";
import illegalSound from "../sounds/illegal.mp3";
import gameStartSound from "../sounds/game-start.mp3";
import promoteSound from "../sounds/promote.mp3";
import castleSound from "../sounds/castle.mp3";

const playSound = (sound: string) => {
  void new Audio(sound).play().catch((error: unknown) => {
    console.error("Error playing move sound:", error);
  });
};

const playMoveSelfSound = () => { playSound(moveSelfSound); };
const playMoveOpponentSound = () => { playSound(moveOpponentSound); };
const playMoveCheckSound = () => { playSound(moveCheckSound); };
const playCaptureSound = () => { playSound(captureSound); };
const playIllegalSound = () => { playSound(illegalSound); };
const playGameStartSound = () => { playSound(gameStartSound); };
const playPromoteSound = () => { playSound(promoteSound); };
const playCastleSound = () => { playSound(castleSound); };

export {
  playMoveSelfSound,
  playMoveOpponentSound,
  playMoveCheckSound,
  playCaptureSound,
  playIllegalSound,
  playGameStartSound,
  playPromoteSound,
  playCastleSound,
};
