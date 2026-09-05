import { t } from '@macro/i18n';
import { cn } from '@ui';
import type { Property } from '../types';

type Props = {
  property: Property;
  class?: string;
};

/**
 * Renders the property's display name — no styling beyond truncate.
 */
export function PropertyLabel(props: Props) {
  return (
    <span class={cn('truncate', props.class)}>
      {t(props.property.displayName)}
    </span>
  );
}
