import { Button } from "@/components/ui/button";
import { ArrowLeftIcon, InfoIcon, SettingsIcon } from "lucide-react";
import { useState } from "react";
import { About } from "./pages/About";
import { SearchSeries } from "./pages/SearchSeries";
import { Settings } from "./pages/Settings";

type View = "search" | "settings" | "about";

export const App = () => {
  const [view, setView] = useState<View>("search");

  return (
    <div className="flex flex-col items-center h-screen w-screen overflow-y-auto p-10 gap-6">
      <div className="relative flex items-center justify-center w-full shrink-0">
        {view === "search" ? (
          <>
            <Button
              variant="ghost"
              size="icon"
              className="absolute left-0 size-12"
              aria-label="Settings"
              onClick={() => setView("settings")}
            >
              <SettingsIcon className="size-5" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="absolute right-0 size-12"
              aria-label="About"
              onClick={() => setView("about")}
            >
              <InfoIcon className="size-5" />
            </Button>
          </>
        ) : (
          <Button
            variant="ghost"
            size="icon"
            className="absolute left-0 size-12"
            aria-label="Back"
            onClick={() => setView("search")}
          >
            <ArrowLeftIcon className="size-5" />
          </Button>
        )}
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
      {view === "search" && <SearchSeries />}
      {view === "settings" && <Settings />}
      {view === "about" && <About />}
    </div>
  );
};
