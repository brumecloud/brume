import { architectureCopy } from "~/architecture/copy";
import { type Tour, tours } from "~/architecture/tours";

export interface TourState {
  readonly id: Tour["id"];
  readonly step: number;
}

interface TourBarProps {
  readonly tour: TourState | null;
  readonly playing: boolean;
  /** False when the visitor prefers reduced motion: the steps then only move by hand. */
  readonly canPlay: boolean;
  readonly onStart: (id: Tour["id"]) => void;
  readonly onStep: (step: number) => void;
  readonly onTogglePlay: () => void;
  readonly onStop: () => void;
}

const control =
  "pixel-border cursor-pointer bg-surface px-3 py-1.5 text-sm transition-colors hover:text-brand disabled:cursor-default disabled:text-ink-dim disabled:hover:text-ink-dim";

/** The tours above the diagram, and the controls of the one in progress. */
export function TourBar({
  tour,
  playing,
  canPlay,
  onStart,
  onStep,
  onTogglePlay,
  onStop,
}: TourBarProps) {
  const current = tour ? tours.find((candidate) => candidate.id === tour.id) : undefined;
  const step = tour && current ? current.steps[tour.step] : undefined;
  const last = current ? current.steps.length - 1 : 0;

  return (
    <div>
      <div className="flex flex-wrap items-center gap-x-4 gap-y-3">
        <p className="text-label text-ink-dim">{architectureCopy.toursLabel}</p>
        <ul className="flex flex-wrap gap-2">
          {tours.map((candidate) => (
            <li key={candidate.id}>
              <button
                type="button"
                aria-pressed={tour?.id === candidate.id}
                onClick={() => onStart(candidate.id)}
                className={`${control} aria-pressed:bg-highlight aria-pressed:text-brand`}
              >
                {candidate.title}
              </button>
            </li>
          ))}
        </ul>
      </div>
      {tour && current && step ? (
        <div className="pixel-border mt-4 flex flex-wrap items-center gap-x-6 gap-y-3 bg-terminal px-5 py-4">
          <p className="text-label shrink-0 text-ink-dim">
            Step {tour.step + 1} of {current.steps.length}
          </p>
          <p
            aria-live="polite"
            className="min-w-0 flex-1 basis-[26rem] leading-relaxed text-pretty"
          >
            {step.caption}
          </p>
          <div className="flex shrink-0 flex-wrap gap-2">
            <button
              type="button"
              disabled={tour.step === 0}
              onClick={() => onStep(tour.step - 1)}
              className={control}
            >
              ← Back
            </button>
            <button
              type="button"
              disabled={tour.step === last}
              onClick={() => onStep(tour.step + 1)}
              className={control}
            >
              Next →
            </button>
            {canPlay ? (
              <button type="button" onClick={onTogglePlay} className={control}>
                {playing ? "Pause" : tour.step === last ? "Replay" : "Play"}
              </button>
            ) : null}
            <button type="button" onClick={onStop} className={control}>
              Stop ×
            </button>
          </div>
        </div>
      ) : null}
    </div>
  );
}
