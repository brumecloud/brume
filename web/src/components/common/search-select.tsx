import { SearchIcon } from "lucide-react";
import type { ReactElement, ReactNode } from "react";

import {
  Combobox,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
  ComboboxPopup,
  ComboboxTrigger,
} from "~/components/ui/combobox";

interface SearchSelectProps<Item> {
  readonly items: readonly Item[];
  readonly value: Item | null;
  readonly onValueChange: (item: Item) => void;
  // The button that opens the list; `children` is its content.
  readonly trigger: ReactElement;
  readonly children: ReactNode;
  // Names the trigger and the list: a combobox does not take its name from its content.
  readonly label: string;
  readonly searchPlaceholder: string;
  readonly emptyText: string;
  readonly itemLabel?: (item: Item) => string;
  readonly isSameItem?: (item: Item, value: Item) => boolean;
}

// A select whose list can be searched: the search field opens with the list.
export function SearchSelect<Item>({
  items,
  value,
  onValueChange,
  trigger,
  children,
  label,
  searchPlaceholder,
  emptyText,
  itemLabel = String,
  isSameItem,
}: SearchSelectProps<Item>) {
  return (
    <Combobox
      items={items}
      value={value}
      itemToStringLabel={itemLabel}
      itemToStringValue={itemLabel}
      {...(isSameItem ? { isItemEqualToValue: isSameItem } : {})}
      onValueChange={(next) => {
        if (next !== null) {
          onValueChange(next);
        }
      }}
    >
      <ComboboxTrigger render={trigger} aria-label={label}>
        {children}
      </ComboboxTrigger>
      <ComboboxPopup className="w-72" aria-label={label}>
        <div className="border-b p-2">
          <ComboboxInput
            size="sm"
            showTrigger={false}
            startAddon={<SearchIcon />}
            placeholder={searchPlaceholder}
            aria-label={searchPlaceholder}
          />
        </div>
        <ComboboxEmpty>{emptyText}</ComboboxEmpty>
        <ComboboxList>
          {(item: Item) => (
            <ComboboxItem key={itemLabel(item)} value={item}>
              {itemLabel(item)}
            </ComboboxItem>
          )}
        </ComboboxList>
      </ComboboxPopup>
    </Combobox>
  );
}
