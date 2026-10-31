import { ConsoleStudio } from "~/components/console-studio";
import { SectionHeading } from "~/components/section";
import { copy } from "~/content";

export function ConsoleDemo() {
  return (
    <section id="console" aria-labelledby="console-title" className="scroll-mt-8 py-24">
      <div className="mx-auto w-full max-w-page px-6">
        <SectionHeading
          id="console-title"
          title={copy.console.title}
          lede={`${copy.console.lede} ${copy.console.invitation}`}
        />
      </div>
      {/* Wider than the text: the shot takes the width of the screen. */}
      <ConsoleStudio />
    </section>
  );
}
