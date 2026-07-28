import { Button } from "@/components/ui/button";
import { ArrowLeftIcon, SettingsIcon } from "lucide-react";
import { useState } from "react";
import { SearchSeries } from "./pages/SearchSeries";
import { Settings } from "./pages/Settings";

export const App = () => {
  const [view, setView] = useState<"search" | "settings">("search");

  return (
    <div className="flex flex-col items-center h-screen w-screen overflow-y-auto p-10 gap-6">
      <div className="relative flex items-center justify-center w-full shrink-0">
        <Button
          variant="ghost"
          size="icon"
          className="absolute left-0 size-12"
          aria-label={view === "search" ? "Settings" : "Back"}
          onClick={() => setView(view === "search" ? "settings" : "search")}
        >
          {view === "search" ? (
            <SettingsIcon className="size-5" />
          ) : (
            <ArrowLeftIcon className="size-5" />
          )}
        </Button>
        <div
          className="text-5xl font-bold text-primary"
          style={{
            textShadow:
              "0 0 24px color-mix(in oklch, var(--primary) 55%, transparent)",
          }}
        >
          Recapp
        </div>
      </div>
      {view === "search" ? <SearchSeries /> : <Settings />}
    </div>
  );
};
