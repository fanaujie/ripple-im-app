/**
 * Date formatting utilities for chat messages
 */

/**
 * Convert timestamp to milliseconds if it's in seconds
 * Backend sends timestamps in UTC seconds, but JavaScript Date expects milliseconds.
 * Timestamps before year 2001 in milliseconds (978307200000) are assumed to be in seconds.
 */
function toMilliseconds(timestamp: number): number {
  // If timestamp is less than 10^12, it's likely in seconds (before year 2001 in ms)
  // 10^12 ms = September 2001, so any reasonable timestamp after 2001 in ms will be > 10^12
  if (timestamp < 1e12) {
    return timestamp * 1000;
  }
  return timestamp;
}

/**
 * Format message date for date separators
 * - Shows "今天" for today
 * - Shows "MM/DD" for this year
 * - Shows "YYYY/MM/DD" for previous years
 */
export function formatMessageDate(timestamp: number | undefined): string {
  // Handle invalid timestamp
  if (typeof timestamp !== 'number' || timestamp <= 0 || isNaN(timestamp)) {
    return 'Unknown';
  }

  try {
    const date = new Date(toMilliseconds(timestamp));

    // Check if date is valid
    if (isNaN(date.getTime())) {
      return 'Unknown';
    }

    const today = new Date();

    // Check if it's today
    if (isSameDay(date, today)) {
      return '今天';
    }

    // Check if it's this year
    if (date.getFullYear() === today.getFullYear()) {
      const month = String(date.getMonth() + 1).padStart(2, '0');
      const day = String(date.getDate()).padStart(2, '0');
      return `${month}/${day}`;
    }

    // Previous years
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${year}/${month}/${day}`;
  } catch (error) {
    console.error('[formatMessageDate] Error formatting timestamp:', timestamp, error);
    return 'Unknown';
  }
}

/**
 * Format message time for message bubbles
 * Shows time in 24-hour format (HH:mm)
 */
export function formatMessageTime(timestamp: number | undefined): string {
  // Handle invalid timestamp
  if (typeof timestamp !== 'number' || timestamp <= 0 || isNaN(timestamp)) {
    return '--:--';
  }

  try {
    const date = new Date(toMilliseconds(timestamp));

    // Check if date is valid
    if (isNaN(date.getTime())) {
      return '--:--';
    }

    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');
    return `${hours}:${minutes}`;
  } catch (error) {
    console.error('[formatMessageTime] Error formatting timestamp:', timestamp, error);
    return '--:--';
  }
}

/**
 * Format conversation last message time for conversation list
 * - Shows "HH:mm" for today
 * - Shows "昨天" for yesterday
 * - Shows "MM/DD" for this year
 * - Shows "YYYY/MM/DD" for previous years
 */
export function formatConversationTime(timestamp: number | undefined): string {
  // Handle invalid timestamp
  if (typeof timestamp !== 'number' || timestamp <= 0 || isNaN(timestamp)) {
    return '--:--';
  }

  try {
    const date = new Date(toMilliseconds(timestamp));

    // Check if date is valid
    if (isNaN(date.getTime())) {
      return '--:--';
    }

    const today = new Date();

    // Check if it's today
    if (isSameDay(date, today)) {
      return formatMessageTime(timestamp);
    }

    // Check if it's yesterday
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);
    if (isSameDay(date, yesterday)) {
      return '昨天';
    }

    // Check if it's this year
    if (date.getFullYear() === today.getFullYear()) {
      const month = String(date.getMonth() + 1).padStart(2, '0');
      const day = String(date.getDate()).padStart(2, '0');
      return `${month}/${day}`;
    }

    // Previous years
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${year}/${month}/${day}`;
  } catch (error) {
    console.error('[formatConversationTime] Error formatting timestamp:', timestamp, error);
    return '--:--';
  }
}

/**
 * Check if two dates are on the same day
 */
function isSameDay(date1: Date, date2: Date): boolean {
  return (
    date1.getFullYear() === date2.getFullYear() &&
    date1.getMonth() === date2.getMonth() &&
    date1.getDate() === date2.getDate()
  );
}

/**
 * Group messages by date
 * Returns a map of date string to messages
 */
export function groupMessagesByDate(
  messages: Array<{ sendTimestamp: number }>
): Map<string, Array<{ sendTimestamp: number }>> {
  const grouped = new Map<string, Array<{ sendTimestamp: number }>>();

  for (const message of messages) {
    const dateKey = formatMessageDate(message.sendTimestamp);
    const existing = grouped.get(dateKey) || [];
    existing.push(message);
    grouped.set(dateKey, existing);
  }

  return grouped;
}
