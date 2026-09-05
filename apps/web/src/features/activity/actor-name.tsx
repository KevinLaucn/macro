import { useUserId } from '@core/context/user';
import { tryMacroId, useDisplayName } from '@core/user';
import type { MacroId } from '@core/user/macroId';
import { t } from '@macro/i18n';
import { Show } from 'solid-js';

/**
 * The actor of an activity row: "You" for the viewer, the display name
 * (local-part email fallback, never the full address) for other users, and
 * "Automation" for bot principals.
 */
export function ActorName(props: { actorId: string }) {
  const userId = useUserId();
  const macroId = () => tryMacroId(props.actorId);

  return (
    <Show when={macroId()} fallback={<>{t('Automation')}</>}>
      {(id) => (
        <Show
          when={props.actorId === userId()}
          fallback={<RemoteName id={id()} />}
        >
          {t('You')}
        </Show>
      )}
    </Show>
  );
}

function RemoteName(props: { id: MacroId }) {
  const [name] = useDisplayName(props.id, { emailFallback: 'local-part' });
  return <>{name()}</>;
}
