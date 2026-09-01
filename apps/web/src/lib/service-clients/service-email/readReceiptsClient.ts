import { SERVER_HOSTS } from '@core/constant/servers';
import { fetchWithToken } from '@core/util/fetchWithToken';

const emailHost: string = SERVER_HOSTS['email-service'];
const EMAIL_LINK_ID_HEADER = 'X-Email-Link-Id';

function emailLinkHeaders(linkId?: string): Record<string, string> | undefined {
  return linkId ? { [EMAIL_LINK_ID_HEADER]: linkId } : undefined;
}

export type ReadReceiptStatus = {
  message_id: string;
  first_opened_at: string | null;
  last_opened_at: string | null;
  open_count: number;
};

export type ReadReceiptStatusesResponse = {
  statuses: ReadReceiptStatus[];
};

export type ReadReceiptsPreferenceResponse = {
  read_receipts_enabled: boolean;
};

export const readReceiptsClient = {
  getStatuses(messageIds: string[]) {
    return fetchWithToken<ReadReceiptStatusesResponse>(
      `${emailHost}/email/messages/tracking`,
      {
        method: 'POST',
        body: JSON.stringify(messageIds),
      }
    );
  },

  getPreference(linkId?: string) {
    return fetchWithToken<ReadReceiptsPreferenceResponse>(
      `${emailHost}/email/settings/read-receipts`,
      {
        method: 'GET',
        headers: emailLinkHeaders(linkId),
      }
    );
  },

  setPreference(readReceiptsEnabled: boolean, linkId?: string) {
    return fetchWithToken<ReadReceiptsPreferenceResponse>(
      `${emailHost}/email/settings/read-receipts`,
      {
        method: 'PATCH',
        headers: emailLinkHeaders(linkId),
        body: JSON.stringify({
          read_receipts_enabled: readReceiptsEnabled,
        }),
      }
    );
  },
};
