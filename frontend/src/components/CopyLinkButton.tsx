import { useState } from "react";
import { useViewportWidth } from "../hooks/useViewportWidth";

import "../css/CopyLinkButton.css";

export const CopyLinkButton: React.FC = () => {
  const [copied, setCopied] = useState(false);

  const { width } = useViewportWidth();

  const copyLinkToClipboard = () => {
    const currentUrl = window.location.href;

    // Remove username parameter, if present
    const url = new URL(currentUrl);
    url.searchParams.delete("username");

    navigator.clipboard
      .writeText(url.toString())
      .then(() => {
        setCopied(true);
        setTimeout(() => {
          setCopied(false);
        }, 1250);
      })
      .catch((err: unknown) => {
        console.error("Failed to copy URL: ", err);
      });
  };

  const showButtonText = width >= 600 || copied;
  const buttonText = copied ? "Copied!" : "Link to join";

  return (
    <button className="copy-link-button" onClick={copyLinkToClipboard}>
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
      {showButtonText && <span>{buttonText}</span>}
    </button>
  );
};
