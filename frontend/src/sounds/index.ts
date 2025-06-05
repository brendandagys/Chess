import moveSelfSound from "../sounds/move-self.mp3";
import moveOpponentSound from "../sounds/move-opponent.mp3";

const playSound = (sound: string) => {
  void new Audio(sound).play().catch((error: unknown) => {
    console.error("Error playing move sound:", error);
  });
};

const playMoveSelfSound = () => { playSound(moveSelfSound); };
const playMoveOpponentSound = () => { playSound(moveOpponentSound); };

export {
  playMoveSelfSound,
  playMoveOpponentSound,
};
