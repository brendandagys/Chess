import moveSelfSound from "../sounds/move-self.mp3";
import moveOpponentSound from "../sounds/move-opponent.mp3";
import moveCheckSound from "../sounds/move-check.mp3";
import captureSound from "../sounds/capture.mp3";
import illegalSound from "../sounds/illegal.mp3";
import gameStartSound from "../sounds/game-start.mp3";
import promoteSound from "../sounds/promote.mp3";
import castleSound from "../sounds/castle.mp3";

// Create Audio instances (one per sound)
const audioMap: Record<string, HTMLAudioElement> = {
  moveSelf: new Audio(moveSelfSound),
  moveOpponent: new Audio(moveOpponentSound),
  moveCheck: new Audio(moveCheckSound),
  capture: new Audio(captureSound),
  illegal: new Audio(illegalSound),
  gameStart: new Audio(gameStartSound),
  promote: new Audio(promoteSound),
  castle: new Audio(castleSound),
};

// Mobile browsers require user interaction to load audio
let audioLoaded = false;

const preloadAudio = () => {
  if (audioLoaded) return;
  audioLoaded = true;

  for (const audio of Object.values(audioMap)) {
    audio.load(); // Browser will fetch and buffer
  }
};

// Attach one-time preloader on first mouse down or touch start
const addPreloadListener = () => {
  const handler = () => {
    preloadAudio();
  };
  window.addEventListener("mousedown", handler, { once: true });
  window.addEventListener("touchstart", handler, { once: true });
};

addPreloadListener();

const playSound = (key: keyof typeof audioMap) => {
  try {
    const audio = audioMap[key];

    if (!audio.paused) {
      audio.currentTime = 0;
    }

    void audio.play();
  } catch (error) {
    console.error("Error playing sound:", error);
  }
};

export const playMoveSelfSound = () => { playSound("moveSelf"); };
export const playMoveOpponentSound = () => { playSound("moveOpponent"); };
export const playMoveCheckSound = () => { playSound("moveCheck"); };
export const playCaptureSound = () => { playSound("capture"); };
export const playIllegalSound = () => { playSound("illegal"); };
export const playGameStartSound = () => { playSound("gameStart"); };
export const playPromoteSound = () => { playSound("promote"); };
export const playCastleSound = () => { playSound("castle"); };
