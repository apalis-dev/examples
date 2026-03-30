import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatTime(date: Date): string {
  const now = new Date();
  const reminderDate = new Date(date);

  // Check if it's today
  const isToday =
    reminderDate.getDate() === now.getDate() &&
    reminderDate.getMonth() === now.getMonth() &&
    reminderDate.getFullYear() === now.getFullYear();

  // Check if it's tomorrow
  const tomorrow = new Date(now);
  tomorrow.setDate(tomorrow.getDate() + 1);
  const isTomorrow =
    reminderDate.getDate() === tomorrow.getDate() &&
    reminderDate.getMonth() === tomorrow.getMonth() &&
    reminderDate.getFullYear() === tomorrow.getFullYear();

  const timeStr = reminderDate.toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit',
    hour12: true,
  });

  if (isToday) return `Today, ${timeStr}`;
  if (isTomorrow) return `Tomorrow, ${timeStr}`;

  return reminderDate.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hour12: true,
  });
}

export function formatFullDateTime(date: Date): string {
  return date.toLocaleDateString('en-US', {
    weekday: 'long',
    month: 'long',
    day: 'numeric',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: true,
  });
}

export function isReminderPassed(scheduledTime: Date): boolean {
  return new Date() > new Date(scheduledTime);
}
