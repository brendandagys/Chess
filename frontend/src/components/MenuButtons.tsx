import { useState, useEffect } from "react";
import { useViewportWidth } from "../hooks/useViewportWidth";
import { useLocalStorage } from "../hooks/useLocalStorage";
import { setSoundsEnabled } from "../sounds";

import "../css/MenuButtons.css";

export const MenuButtons: React.FC = () => {
  const [soundsPref, setSoundsPref] = useLocalStorage("sounds-enabled", "true");
  const soundsOn = soundsPref !== "false";

  const [justCopied, setJustCopied] = useState(false);
  const [justToggledSoundPreference, setJustToggledSoundPreference] =
    useState(false);

  const { width } = useViewportWidth();

  useEffect(() => {
    setSoundsEnabled(soundsOn);
  }, [soundsOn]);

  const copyLinkToClipboard = () => {
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
    setSoundsPref(soundsOn ? "false" : "true");

    setJustToggledSoundPreference(true);
    setTimeout(() => {
      setJustToggledSoundPreference(false);
    }, 1250);
  };

  const showCopyButtonText = width >= 600 || justCopied;
  const showSoundButtonText = width >= 600 || justToggledSoundPreference;

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
            <span>{justCopied ? "Copied!" : "Link to join"}</span>
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
    </div>
  );
};
