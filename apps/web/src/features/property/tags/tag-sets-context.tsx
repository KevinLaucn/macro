import { useTagsQuery } from '@queries/properties/tags';
import type { PropertyOptionResponse } from '@service-properties/generated/schemas/propertyOptionResponse';
import type { TagScope } from '@service-properties/generated/schemas/tagScope';
import type { TagSetResponse } from '@service-properties/generated/schemas/tagSetResponse';
import {
  type Accessor,
  createContext,
  createMemo,
  type FlowComponent,
  useContext,
} from 'solid-js';

type TagSets = Accessor<TagSetResponse[]>;
type TagOption = { option: PropertyOptionResponse; scope: TagScope };
type TagSetsContextValue = {
  tagSets: TagSets;
  optionById: Accessor<ReadonlyMap<string, TagOption>>;
};

const TagSetsContext = createContext<TagSetsContextValue>();

/** Shares loaded tag definitions with a feature subtree. */
export const TagSetsProvider: FlowComponent<{ tagSets: TagSets }> = (props) => {
  const safeTagSets = (): TagSetResponse[] => {
    try {
      const res = props.tagSets();
      return Array.isArray(res) ? res : [];
    } catch {
      return [];
    }
  };

  const optionById = createMemo(() => {
    const options = new Map<string, TagOption>();
    const sets = safeTagSets();
    for (const set of sets) {
      if (Array.isArray(set?.options)) {
        for (const option of set.options) {
          if (option?.id) {
            options.set(option.id, { option, scope: set.scope });
          }
        }
      }
    }
    return options;
  });

  return (
    <TagSetsContext.Provider value={{ tagSets: safeTagSets, optionById }}>
      {props.children}
    </TagSetsContext.Provider>
  );
};

/** Explicit query-owning adapter for standalone tag-aware lists. */
export const TagSetsQueryProvider: FlowComponent = (props) => {
  const tagsQuery = useTagsQuery();
  const tagSets = (): TagSetResponse[] =>
    Array.isArray(tagsQuery.data) ? tagsQuery.data : [];

  return <TagSetsProvider tagSets={tagSets}>{props.children}</TagSetsProvider>;
};

function useTagSetsContext(): TagSetsContextValue {
  const context = useContext(TagSetsContext);
  if (!context) {
    throw new Error('useTagSets can only be used under a TagSetsProvider');
  }

  return context;
}

/** Returns tag definitions from the nearest metadata provider. */
export function useTagSets(): TagSets {
  return useTagSetsContext().tagSets;
}

/** Returns the provider-owned tag-option index shared by every list row. */
export function useTagOptionById() {
  return useTagSetsContext().optionById;
}
