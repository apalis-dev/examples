import { ReminderProvider } from '@/lib/reminder-context';
import { ReminderSidebar } from '@/components/reminder-sidebar';
import { ReminderEditor } from '@/components/reminder-editor';
import "./App.css";

export default function App() {



  return (
    <ReminderProvider>
      <div className="flex h-screen w-full">
        <div className="hidden lg:flex w-80 flex-shrink-0">
          <ReminderSidebar />
        </div>
        <div className="flex-1 overflow-hidden">
          <ReminderEditor />
        </div>
      </div>
    </ReminderProvider>
  );
}
