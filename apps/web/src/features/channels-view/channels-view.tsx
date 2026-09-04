import { onMount } from 'solid-js';
import { useSplitPanelOrThrow } from '@components/app/split-layout/layoutUtils';
import ChatCircleDotsIcon from '@phosphor/chat-circle-dots.svg';

export function ChannelsView() {
  const panel = useSplitPanelOrThrow();

  onMount(() => {
    panel.handle.setDisplayName('Channels');
  });

  return (
    <div class="flex size-full items-center justify-center p-8 bg-surface">
      <div class="flex max-w-md flex-col items-center text-center gap-4">
        <div class="flex size-16 items-center justify-center rounded-2xl bg-surface-muted text-accent">
          <ChatCircleDotsIcon class="size-8" />
        </div>
        <div class="flex flex-col gap-1.5">
          <h2 class="text-xl font-semibold text-ink">
            Omnichannel Communications
          </h2>
          <p class="text-sm text-ink-muted leading-relaxed">
            Multi-platform messaging hub is coming soon. Connect and manage customer conversations across WhatsApp, Instagram, Facebook Messenger, and more in one unified interface.
          </p>
        </div>
        <div class="flex flex-wrap justify-center gap-2 mt-2">
          <span class="px-2.5 py-1 text-xs font-medium rounded-full bg-surface-muted text-ink-muted">
            WhatsApp
          </span>
          <span class="px-2.5 py-1 text-xs font-medium rounded-full bg-surface-muted text-ink-muted">
            Instagram
          </span>
          <span class="px-2.5 py-1 text-xs font-medium rounded-full bg-surface-muted text-ink-muted">
            Messenger
          </span>
        </div>
      </div>
    </div>
  );
}
