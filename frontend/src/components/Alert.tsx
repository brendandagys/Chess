import { GameMessage } from "../types/sharedComponentTypes";

import "../css/Alert.css";

interface AlertProps {
  message: GameMessage;
  onDismiss: () => void;
}

export const Alert: React.FC<AlertProps> = ({ message, onDismiss }) => {
  return (
    <div className={`alert alert--${message.errorType}`}>
      <span className="message">{message.message}</span>
      <button className="alert-dismiss" onClick={onDismiss}>
        &times;
      </button>
    </div>
  );
};
