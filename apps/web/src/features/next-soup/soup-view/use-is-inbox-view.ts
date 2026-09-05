import { isListViewID } from '@app/constants/list-views';
import { useSplitPanelOrThrow } from '@components/app/split-layout/layoutUtils';

export function useIsInboxView() {
  const panel = useSplitPanelOrThrow();

  const currentView = () => {
    const content = panel.handle.content();
    if (!content || content.type !== 'component') return;
    return isListViewID(content.id) ? content.id : undefined;
  };

  return () => currentView() === 'inbox';
}
