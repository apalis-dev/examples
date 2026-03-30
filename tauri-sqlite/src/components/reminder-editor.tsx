'use client';

import { useReminders } from '@/lib/reminder-context';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Clock, AlertCircle, CheckCircle2 } from 'lucide-react';
import { isReminderPassed, formatFullDateTime } from '@/lib/utils';

export function ReminderEditor() {
  const { reminders, selectedReminderId, updateReminder } = useReminders();

  const selectedReminder = reminders.find((r) => r.id === selectedReminderId);

  if (!selectedReminder) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center">
          <Clock className="mx-auto h-12 w-12 text-muted-foreground/40 mb-4" />
          <p className="text-lg font-medium text-foreground">Select a reminder</p>
          <p className="text-sm text-muted-foreground mt-2">Choose one from the sidebar to view details</p>
        </div>
      </div>
    );
  }

  const isPassed = isReminderPassed(selectedReminder.scheduledTime);

  const handleTitleChange = (value: string) => {
    updateReminder(selectedReminder.id, { title: value });
  };

  const handleTimeChange = (value: string) => {
    if (!value) return;
    updateReminder(selectedReminder.id, { scheduledTime: new Date(value) });
  };

  const handleReminderTextChange = (value: string) => {
    updateReminder(selectedReminder.id, { reminderText: value });
  };

  const handleNoteChange = (value: string) => {
    updateReminder(selectedReminder.id, { note: value });
  };

  const handleToggleComplete = () => {
    updateReminder(selectedReminder.id, { isComplete: !selectedReminder.isComplete });
  };

  return (
    <div className="h-full flex flex-col bg-background p-8">
      {/* Header with Status */}
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-4">
          {selectedReminder.isComplete ? (
            <button
              onClick={handleToggleComplete}
              className="flex-shrink-0 text-green-600 hover:text-green-700 transition-colors"
            >
              <CheckCircle2 className="h-6 w-6" />
            </button>
          ) : isPassed ? (
            <div className="flex-shrink-0 text-orange-500">
              <AlertCircle className="h-6 w-6" />
            </div>
          ) : (
            <button
              onClick={handleToggleComplete}
              className="flex-shrink-0 text-primary hover:text-primary/80 transition-colors"
            >
              <Clock className="h-6 w-6" />
            </button>
          )}
          <span className="text-sm font-medium text-muted-foreground">
            {selectedReminder.isComplete
              ? 'Completed'
              : isPassed
                ? 'Time passed'
                : 'Upcoming'}
          </span>
        </div>

        {/* Title */}
        <Input
          value={selectedReminder.title}
          onChange={(e) => handleTitleChange(e.target.value)}
          placeholder="Reminder title"
          className="text-3xl font-bold h-auto py-2 px-0 border-0 border-b border-border focus-visible:ring-0 focus-visible:border-primary"
        />
      </div>

      {/* Main Content */}
      <div className="flex-1 overflow-y-auto space-y-8 pb-8">
        {/* Schedule Time */}
        <div className="space-y-2">
          <label className="text-sm font-medium text-foreground flex items-center gap-2">
            <Clock className="h-4 w-4" />
            Schedule Time
          </label>
          <Input
            type="datetime-local"
            value={new Date(selectedReminder.scheduledTime).toISOString().slice(0, 16)}
            onChange={(e) => handleTimeChange(e.target.value)}
            className="text-base"
          />
          <p className="text-xs text-muted-foreground">
            {formatFullDateTime(new Date(selectedReminder.scheduledTime))}
          </p>
        </div>

        {/* Reminder Text */}
        <div className="space-y-2">
          <label className="text-sm font-medium text-foreground">Reminder Text</label>
          <Textarea
            value={selectedReminder.reminderText}
            onChange={(e) => handleReminderTextChange(e.target.value)}
            placeholder="What do you want to be reminded about?"
            className="min-h-32 resize-none text-base"
          />
        </div>

        {/* Note - Shown after reminder passes */}
        {isPassed && (
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <label className="text-sm font-medium text-foreground">Note</label>
              <span className="text-xs bg-orange-100 text-orange-800 px-2 py-1 rounded">
                Added after reminder passed
              </span>
            </div>
            <Textarea
              value={selectedReminder.note}
              onChange={(e) => handleNoteChange(e.target.value)}
              placeholder="Add any notes or follow-up information..."
              className="min-h-32 resize-none text-base"
            />
          </div>
        )}

        {/* Show note field even if not passed, but with hint */}
        {!isPassed && (
          <div className="space-y-2 opacity-50">
            <label className="text-sm font-medium text-foreground">Note</label>
            <p className="text-xs text-muted-foreground italic">
              Note field will be available after the reminder time passes
            </p>
            <Textarea
              value={selectedReminder.note}
              onChange={(e) => handleNoteChange(e.target.value)}
              placeholder="Add any notes or follow-up information..."
              className="min-h-32 resize-none text-base"
              disabled
            />
          </div>
        )}
      </div>
    </div>
  );
}
