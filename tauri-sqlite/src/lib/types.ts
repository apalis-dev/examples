export interface Reminder {
    id: string;
    title: string;
    scheduledTime: Date;
    reminderText: string;
    note: string;
    isComplete: boolean;
    createdAt: Date;
  }
  