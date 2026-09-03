// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import {
  formatSeenLabel,
  removeOwnTrackingPixels,
  stripOwnTrackingPixelsFromHtml,
} from './readReceipts';

const TOKEN = 'open-tracking-token-test';

describe('removeOwnTrackingPixels', () => {
  it('removes Macro open tracking images and keeps ordinary images', () => {
    const doc = new DOMParser().parseFromString(
      `<p>hi</p><img src="https://email-service.macro.com/t/o/${TOKEN}"><img src="https://example.com/logo.png">`,
      'text/html'
    );

    removeOwnTrackingPixels(doc);

    expect(doc.querySelectorAll('img')).toHaveLength(1);
    expect(doc.querySelector('img')?.getAttribute('src')).toBe(
      'https://example.com/logo.png'
    );
  });
});

describe('stripOwnTrackingPixelsFromHtml', () => {
  it('strips the receipt before sent-mail HTML is rendered', () => {
    const html = `<p>sent</p><img src="https://email-service-dev.macro.com/t/o/${TOKEN}" width="1" height="1">`;
    const stripped = stripOwnTrackingPixelsFromHtml(html);

    expect(stripped).toContain('<p>sent</p>');
    expect(stripped).not.toContain('/t/o/');
  });
});

describe('formatSeenLabel', () => {
  it('formats recent opens relative to now', () => {
    expect(formatSeenLabel(new Date())).toBe('Seen just now');
    expect(formatSeenLabel(new Date(Date.now() - 5 * 60_000))).toBe(
      'Seen 5m ago'
    );
  });
});
