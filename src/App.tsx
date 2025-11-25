import "./App.css";

import { NotificationCenter } from "@/components/app/notification-center";
import { TabWorkspace } from "@/features/tab/components/tab-workspace";

function App() {
  return (
    <>
      <NotificationCenter />
      <TabWorkspace />
    </>
  );
}

export default App;
