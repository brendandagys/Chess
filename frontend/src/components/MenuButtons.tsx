import { useState, useEffect, useRef } from "react";
import { useViewportWidth } from "../hooks/useViewportWidth";
import { useLocalStorage } from "../hooks/useLocalStorage";
import { setSoundsEnabled } from "../sounds";

import "../css/MenuButtons.css";

interface MenuButtonsProps {
  realismOn: boolean;
  setRealismPref: React.Dispatch<React.SetStateAction<string>>;
  evalOn: boolean;
  setEvalPref: React.Dispatch<React.SetStateAction<string>>;
}

const EXPANDED_AUTO_CLOSE_MS = 5000;
const POST_ACTION_CLOSE_MS = 2500;

export const MenuButtons: React.FC<MenuButtonsProps> = ({
  realismOn,
  setRealismPref,
  evalOn,
  setEvalPref,
}) => {
  const [soundsPref, setSoundsPref] = useLocalStorage("sounds-enabled", "true");
  const soundsOn = soundsPref !== "false";

  const [justCopied, setJustCopied] = useState(false);
  const [justToggledSoundPreference, setJustToggledSoundPreference] =
    useState(false);
  const [justToggledRealismPreference, setJustToggledRealismPreference] =
    useState(false);
  const [justToggledEvalPreference, setJustToggledEvalPreference] =
    useState(false);

  const [isExpanded, setIsExpanded] = useState(false);
  const collapseTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const { width } = useViewportWidth();

  useEffect(() => {
    setSoundsEnabled(soundsOn);
  }, [soundsOn]);

  useEffect(() => {
    return () => {
      if (collapseTimerRef.current) clearTimeout(collapseTimerRef.current);
    };
  }, []);

  const scheduleCollapse = (ms: number) => {
    if (collapseTimerRef.current) clearTimeout(collapseTimerRef.current);
    collapseTimerRef.current = setTimeout(() => {
      setIsExpanded(false);
    }, ms);
  };

  const handleSettingsClick = () => {
    setIsExpanded(true);
    scheduleCollapse(EXPANDED_AUTO_CLOSE_MS);
  };

  const afterAction = () => {
    scheduleCollapse(POST_ACTION_CLOSE_MS);
  };

  const copyLinkToClipboard = () => {
    afterAction();
    const currentUrl = window.location.href;

    // Remove username parameter, if present
    const url = new URL(currentUrl);
    url.searchParams.delete("username");

    navigator.clipboard
      .writeText(url.toString())
      .then(() => {
        setJustCopied(true);
        setTimeout(() => {
          setJustCopied(false);
        }, 1250);
      })
      .catch((err: unknown) => {
        console.error("Failed to copy URL: ", err);
      });
  };

  const toggleSoundPreference = () => {
    afterAction();
    setSoundsPref(soundsOn ? "false" : "true");

    setJustToggledSoundPreference(true);
    setTimeout(() => {
      setJustToggledSoundPreference(false);
    }, 1250);
  };

  const toggleRealismPreference = () => {
    afterAction();
    setRealismPref(realismOn ? "false" : "true");

    setJustToggledRealismPreference(true);
    setTimeout(() => {
      setJustToggledRealismPreference(false);
    }, 1250);
  };

  const toggleEvalPreference = () => {
    afterAction();
    setEvalPref(evalOn ? "false" : "true");

    setJustToggledEvalPreference(true);
    setTimeout(() => {
      setJustToggledEvalPreference(false);
    }, 1250);
  };

  const breakpoint = 850;

  const showCopyButtonText = width >= breakpoint || justCopied;
  const showSoundButtonText = width >= breakpoint || justToggledSoundPreference;
  const showRealismButtonText =
    width >= breakpoint || justToggledRealismPreference;
  const showEvalButtonText = width >= breakpoint || justToggledEvalPreference;

  if (!isExpanded) {
    return (
      <div className="menu-buttons menu-buttons--settings-toggle">
        <button
          className="menu-button settings-toggle-button"
          onClick={handleSettingsClick}
          title="Settings"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            width="16"
            height="16"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <circle cx="12" cy="12" r="3" />
            {/* eslint-disable-next-line max-len */}
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
        </button>
      </div>
    );
  }

  return (
    <div>
      <div className="menu-buttons menu-buttons--copy">
        <button
          className="menu-button copy-link-button"
          onClick={copyLinkToClipboard}
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            width="16"
            height="16"
          >
            <path
              d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
            />
            <path
              d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
            />
          </svg>
          {showCopyButtonText && (
            <span>{justCopied ? "Copied!" : "Copy link"}</span>
          )}
        </button>
      </div>

      <div className="menu-buttons menu-buttons--sound">
        <button
          className={`menu-button sound-toggle-button${
            !soundsOn ? " sound-toggle-button--off" : ""
          }`}
          onClick={() => {
            toggleSoundPreference();
          }}
        >
          {soundsOn ? (
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 24 24"
              width="16"
              height="16"
            >
              <polygon
                points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
              <path
                d="M19.07 4.93a10 10 0 0 1 0 14.14"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
              />
              <path
                d="M15.54 8.46a5 5 0 0 1 0 7.07"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
              />
            </svg>
          ) : (
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 24 24"
              width="16"
              height="16"
            >
              <polygon
                points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
              <line
                x1="23"
                y1="9"
                x2="17"
                y2="15"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
              />
              <line
                x1="17"
                y1="9"
                x2="23"
                y2="15"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
              />
            </svg>
          )}
          {showSoundButtonText && (
            <span>{soundsOn ? "Sound on" : "Sound off"}</span>
          )}
        </button>
      </div>

      <div className="menu-buttons menu-buttons--realism">
        <button
          className={`menu-button realism-toggle-button${
            !realismOn ? " realism-toggle-button--off" : ""
          }`}
          onClick={toggleRealismPreference}
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            width="16"
            height="16"
          >
            <circle
              cx="12"
              cy="12"
              r="10"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            />
            <polyline
              points="12 6 12 12 16 14"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
          {showRealismButtonText && (
            <span>{realismOn ? "Realism" : "No delay"}</span>
          )}
        </button>
      </div>

      <div className="menu-buttons menu-buttons--eval">
        <button
          className={`menu-button eval-toggle-button${
            !evalOn ? " eval-toggle-button--off" : ""
          }`}
          onClick={toggleEvalPreference}
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            width="16"
            height="16"
          >
            <rect
              x="2"
              y="14"
              width="4"
              height="8"
              rx="1"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinejoin="round"
            />
            <rect
              x="9"
              y="9"
              width="4"
              height="13"
              rx="1"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinejoin="round"
            />
            <rect
              x="16"
              y="4"
              width="4"
              height="18"
              rx="1"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinejoin="round"
            />
          </svg>
          {showEvalButtonText && <span>{evalOn ? "Eval on" : "Eval off"}</span>}
        </button>
      </div>
    </div>
  );
};
