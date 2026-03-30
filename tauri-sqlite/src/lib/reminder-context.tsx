'use client';

import React, { createContext, useContext, useState, useEffect } from 'react';
import { Reminder } from './types';

import { invoke } from "@tauri-apps/api/core";


interface ReminderContextType {
  reminders: Reminder[];
  selectedReminderId: number | null;
  addReminder: (reminder: Omit<Reminder, 'id' | 'createdAt'>) => void;
  updateReminder: (id: number, reminder: Partial<Reminder>) => void;
  deleteReminder: (id: number) => void;
  selectReminder: (id: number | null) => void;
}

const ReminderContext = createContext<ReminderContextType | undefined>(undefined);

export function ReminderProvider({ children }: { children: React.ReactNode }) {
  const [reminders, setReminders] = useState<Reminder[]>([]);
  const [selectedReminderId, setSelectedReminderId] = useState<number | null>(null);
  const [isLoaded, setIsLoaded] = useState(false);

  const fetchReminders = async () => {
    let r = await invoke("fetch_reminders") as Reminder[];
    setReminders(r)
    setIsLoaded(true)
  };

  // Load from localStorage on mount
  useEffect(() => {
    fetchReminders().then(res => {
      if (reminders.length > 0) {
        setSelectedReminderId(reminders[0].id);
      }
    })
  }, []);

  // Save to localStorage whenever reminders change
  useEffect(() => {
    if (!isLoaded) {
      fetchReminders()
    }
  }, [isLoaded]);

  const addReminder = async (reminder: Omit<Reminder, 'id' | 'createdAt'>) => {
    let newId = await invoke("add_reminder", { reminder });
    setSelectedReminderId(newId as number);
    setIsLoaded(false)
  };

  const updateReminder = async (id: number, updates: Partial<Reminder>) => {
    await invoke("update_reminder", { id, reminder: updates });
    setIsLoaded(false)
  };

  const deleteReminder = async (id: number) => {
    await invoke("delete_reminder", { id });
    setIsLoaded(false)
  };

  const selectReminder = (id: number | null) => {
    setSelectedReminderId(id);
  };

  return (
    <ReminderContext.Provider
      value={{
        reminders,
        selectedReminderId,
        addReminder,
        updateReminder,
        deleteReminder,
        selectReminder,
      }}
    >
      {children}
    </ReminderContext.Provider>
  );
}

export function useReminders() {
  const context = useContext(ReminderContext);
  if (!context) {
    throw new Error('useReminders must be used within ReminderProvider');
  }
  return context;
}
