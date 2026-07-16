import { Children, isValidElement, type PropsWithChildren, type ReactElement } from "react";

type TabProps = PropsWithChildren<{ title: string }>;

export function Tab({ children }: TabProps) {
  return <>{children}</>;
}

export function Tabs({ children }: PropsWithChildren) {
  const tabs = Children.toArray(children).filter(
    (child): child is ReactElement<TabProps> => isValidElement<TabProps>(child),
  );

  return (
    <section className="brume-tabs" data-brume-tabs>
      <div className="brume-tab-list" role="tablist">
        {tabs.map((tab, index) => (
          <button
            aria-selected={index === 0}
            className="brume-tab-button"
            data-brume-tab-button={index}
            key={`${tab.props.title}-${index}`}
            role="tab"
            type="button"
          >
            {tab.props.title}
          </button>
        ))}
      </div>
      {tabs.map((tab, index) => (
        <div
          className="brume-tab-panel"
          data-brume-tab-panel={index}
          hidden={index !== 0}
          key={`${tab.props.title}-${index}`}
          role="tabpanel"
        >
          {tab.props.children}
        </div>
      ))}
    </section>
  );
}

