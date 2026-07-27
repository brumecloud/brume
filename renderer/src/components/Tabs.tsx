import {
  Children,
  isValidElement,
  type PropsWithChildren,
  type ReactElement,
} from "react";

type TabProps = PropsWithChildren<{ title: string; value?: string }>;

export function Tab({ children }: TabProps) {
  return <>{children}</>;
}

export function Tabs({ children }: PropsWithChildren) {
  const tabs = Children.toArray(children).filter(
    (child): child is ReactElement<TabProps> => isValidElement<TabProps>(child),
  );

  const tabId = tabs
    .map((tab) => tab.props.value ?? tab.props.title)
    .join("-")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");

  return (
    <section className="brume-tabs" data-brume-tabs id={tabId ? `tabs-${tabId}` : undefined}>
      <div className="brume-tab-list" role="tablist">
        {tabs.map((tab, index) => (
          <button
            aria-selected={index === 0}
            aria-controls={`${tabId || "tabs"}-panel-${index}`}
            className="brume-tab-button"
            data-brume-tab-button={index}
            id={`${tabId || "tabs"}-tab-${index}`}
            key={`${tab.props.title}-${index}`}
            role="tab"
            tabIndex={index === 0 ? 0 : -1}
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
          id={`${tabId || "tabs"}-panel-${index}`}
          key={`${tab.props.title}-${index}`}
          role="tabpanel"
          aria-labelledby={`${tabId || "tabs"}-tab-${index}`}
          tabIndex={0}
        >
          {tab.props.children}
        </div>
      ))}
    </section>
  );
}
