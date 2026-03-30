'use client';

import { useReminders } from '@/lib/reminder-context';
import { Button } from '@/components/ui/button';
import { Clock, Plus, Trash2, CheckCircle2, Circle } from 'lucide-react';
import { formatTime, isReminderPassed } from '@/lib/utils';

export function ReminderSidebar() {
  const { reminders, selectedReminderId, selectReminder, deleteReminder, addReminder } =
    useReminders();

  const handleNewReminder = () => {
    addReminder({
      title: 'New Reminder',
      scheduledTime: new Date(Date.now() + 60000),
      reminderText: '',
      note: '',
      isComplete: false,
    });
  };

  return (
    <div className="flex h-full flex-col border-r border-border bg-sidebar text-sidebar-foreground">
      {/* Header */}
      <div className="border-b border-sidebar-border px-6 py-5">
        <h1 className="text-xl font-semibold text-sidebar-foreground">Reminders</h1>
      </div>

      {/* Add Button */}
      <div className="px-4 py-4">
        <Button
          onClick={handleNewReminder}
          className="w-full bg-sidebar-primary text-sidebar-primary-foreground hover:bg-sidebar-primary/90 gap-2"
        >
          <Plus className="h-4 w-4" />
          New Reminder
        </Button>
      </div>

      {/* Reminders List */}
      <div className="flex-1 overflow-y-auto">
        {reminders.length === 0 ? (
          <div className="flex items-center justify-center px-6 py-12">
            <p className="text-sm text-sidebar-accent-foreground/60">No reminders yet</p>
          </div>
        ) : (
          <div className="space-y-2 px-4 pb-4">
            {reminders.map((reminder) => {
              const isPassed = isReminderPassed(reminder.scheduledTime);
              const isSelected = reminder.id === selectedReminderId;

              return (
                <div
                  key={reminder.id}
                  onClick={() => selectReminder(reminder.id)}
                  className={`w-full rounded-lg px-4 py-3 text-left transition-colors cursor-pointer ${
                    isSelected
                      ? 'bg-sidebar-primary/10 text-sidebar-primary'
                      : 'hover:bg-sidebar-accent/50 text-sidebar-foreground'
                  }`}
                  role="button"
                  tabIndex={0}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      selectReminder(reminder.id);
                    }
                  }}
                >
                  <div className="flex items-start gap-3">
                    <div className="mt-1">
                      {reminder.isComplete ? (
                        <CheckCircle2 className="h-4 w-4 text-green-600" />
                      ) : isPassed ? (
                        <Circle className="h-4 w-4 fill-orange-500 text-orange-500" />
                      ) : (
                        <Clock className="h-4 w-4 text-sidebar-primary" />
                      )}
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium truncate">{reminder.title}</p>
                      <p className="text-xs text-sidebar-accent-foreground/60 mt-1">
                        {formatTime(reminder.scheduledTime)}
                      </p>
                    </div>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        deleteReminder(reminder.id);
                      }}
                      className="flex-shrink-0 text-sidebar-accent-foreground/40 hover:text-red-500 transition-colors p-1"
                      aria-label="Delete reminder"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
