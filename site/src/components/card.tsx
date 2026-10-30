interface CardProps {
  readonly label: string;
  readonly title: string;
  readonly text: string;
}

export function Card({ label, title, text }: CardProps) {
  return (
    <article className="pixel-border bg-surface p-6">
      <p className="text-pixel text-ink-dim">{label}</p>
      <h3 className="mt-4 text-[17px] leading-snug font-semibold tracking-[-0.015em]">{title}</h3>
      <p className="mt-2.5 text-[15px] leading-relaxed text-ink-muted">{text}</p>
    </article>
  );
}
