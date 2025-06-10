import { FormToShow } from "@src/types/sharedComponentTypes";

import hero from "@src/images/hero.png";

interface HeaderSectionProps {
  setFormToShow: React.Dispatch<React.SetStateAction<FormToShow>>;
  setShowForm: React.Dispatch<React.SetStateAction<boolean>>;
}

export const HeaderSection: React.FC<HeaderSectionProps> = ({
  setFormToShow,
  setShowForm,
}) => {
  return (
    <>
      <div className="title-container">
        <img
          src={hero}
          alt="Play Chess"
          className="hero-image"
          onClick={() => {
            window.location.href = "/";
          }}
        />
      </div>

      <div className="sub-title-container">
        <button
          onClick={() => {
            setFormToShow(FormToShow.Create);
            setShowForm(true);
          }}
          className="main-action-button main-action-button--secondary"
        >
          Start new game
        </button>
        <button
          onClick={() => {
            setFormToShow(FormToShow.Join);
            setShowForm(true);
          }}
          className="main-action-button main-action-button"
        >
          Join existing game
        </button>
      </div>
    </>
  );
};
