import { useRouterState } from "@tanstack/react-router";
import { useState } from "react";
import {
  SquarePen,
  Clock,
  Search,
  Library,
  FolderPlus,
  Plus,
  ListFilter,
  PanelLeft,
  Settings,
  MessageSquareDashed,
  LoaderCircle,
} from "lucide-react";
import { cn } from "@/lib/utils";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

const nav = [
  { title: "New task", id: "new", icon: SquarePen, badge: null as string | null },
  { title: "Scheduled", id: "scheduled", icon: Clock, badge: null },
  { title: "Search", id: "search", icon: Search, badge: null },
  { title: "Library", id: "library", icon: Library, badge: null },
];

type AppSidebarProps = {
  activeView?: string;
  currentTaskTitle?: string | null;
  projects?: { id: string; name: string; threads: string[] }[];
  onCreateProject?: () => void;
  onCreateThread?: (projectId: string) => void;
  onNewTask?: () => void;
  onSearch?: () => void;
  onSettings?: () => void;
  onToggle?: () => void;
  onViewChange?: (view: string) => void;
};

export function AppSidebar({
  activeView = "new",
  projects = [],
  currentTaskTitle,
  onCreateProject,
  onCreateThread,
  onNewTask,
  onSearch,
  onSettings,
  onToggle,
  onViewChange,
}: AppSidebarProps) {
  const path = useRouterState({ select: (r) => r.location.pathname });
  const [taskFilter, setTaskFilter] = useState("All");
  const isActive = (id: string) => path === "/" && activeView === id;

  return (
    <aside className="flex h-screen w-[260px] shrink-0 flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground">
      {/* Brand */}
      <div className="flex items-center justify-between px-4 pt-4 pb-2">
        <div className="flex items-center gap-2">
          <div className="grid size-7 place-items-center rounded-md bg-sidebar-accent text-sidebar-accent-foreground">
            <span className="text-base">✦</span>
          </div>
          <span className="font-serif text-2xl tracking-tight">Onyx</span>
        </div>
        <button
          onClick={onToggle}
          className="grid size-7 place-items-center rounded-md text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-foreground"
          aria-label="Collapse sidebar"
        >
          <PanelLeft className="size-4" />
        </button>
      </div>

      {/* Nav */}
      <nav className="mt-2 px-2">
        {nav.map((item) => {
          const Icon = item.icon;
          const active = isActive(item.id);
          return (
            <button
              key={item.title}
              onClick={() => {
                if (item.id === "new") {
                  onNewTask?.();
                } else if (item.id === "search") {
                  onSearch?.();
                } else {
                  onViewChange?.(item.id);
                }
              }}
              className={cn(
                "group flex w-full items-center gap-3 rounded-md px-3 py-2 text-left text-[13px] font-medium transition-colors",
                active
                  ? "bg-sidebar-primary text-sidebar-primary-foreground"
                  : "text-sidebar-foreground/85 hover:bg-sidebar-accent",
              )}
            >
              <Icon className="size-4" />
              <span className="flex-1">{item.title}</span>
              {item.badge && (
                <span className="rounded-md bg-primary/20 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-primary">
                  {item.badge}
                </span>
              )}
            </button>
          );
        })}
      </nav>

      {/* Projects */}
      <div className="mt-6 px-2">
        <div className="flex items-center justify-between px-3 pb-1">
          <span className="text-xs text-muted-foreground uppercase tracking-wider font-semibold">Projects</span>
          <button
            onClick={onCreateProject}
            className="text-muted-foreground hover:text-sidebar-foreground transition-colors"
            aria-label="Add project"
          >
            <Plus className="size-3.5" />
          </button>
        </div>
        
        <div className="space-y-1 mt-1">
          {projects.map((project) => (
            <div key={project.id} className="group flex flex-col px-1">
              <button
                className="flex w-full items-center gap-3 rounded-md px-2 py-1.5 text-[13px] text-sidebar-foreground/85 hover:bg-sidebar-accent transition-colors"
              >
                <Library className="size-4 text-primary/70" />
                <span className="truncate flex-1 text-left">{project.name}</span>
              </button>
              
              <div className="relative ml-5 mt-1 flex flex-col gap-0.5 border-l border-white/30 pb-3 pl-4">
                {project.threads.map((thread, i) => (
                  <div
                    key={i}
                    className="group/thread relative flex items-center before:absolute before:-left-4 before:top-0 before:h-1/2 before:w-3 before:rounded-bl-lg before:border-b before:border-l before:border-white/35"
                  >
                    <button className="w-full truncate rounded-md px-2 py-1 text-left text-[12.5px] text-muted-foreground transition-colors hover:bg-sidebar-accent/50 hover:text-sidebar-foreground">
                      {thread}
                    </button>
                  </div>
                ))}
                <div className="relative mt-1 flex items-center before:absolute before:-left-4 before:top-0 before:h-1/2 before:w-3 before:rounded-bl-lg before:border-b before:border-l before:border-white/35">
                  <button
                    onClick={() => onCreateThread?.(project.id)}
                    className="flex w-full items-center gap-2 rounded-md px-2 py-1 text-[12px] font-medium text-primary/80 transition-colors hover:bg-primary/10 hover:text-primary"
                  >
                    <Plus className="size-3.5" />
                    <span>Create a thread</span>
                  </button>
                </div>
              </div>
            </div>
          ))}
          
          {projects.length === 0 && (
            <button
              onClick={onCreateProject}
              className="flex w-full items-center gap-3 rounded-md px-3 py-2 text-[13px] text-sidebar-foreground/60 hover:bg-sidebar-accent hover:text-sidebar-foreground transition-colors"
            >
              <FolderPlus className="size-4" />
              <span>New project</span>
            </button>
          )}
        </div>
      </div>

      {/* All tasks */}
      <div className="mt-4 flex items-center justify-between px-5">
        <span className="text-xs text-muted-foreground">{taskFilter} tasks</span>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <button
              className="text-muted-foreground hover:text-sidebar-foreground"
              aria-label="Filter tasks"
            >
              <ListFilter className="size-3.5" />
            </button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start">
            <DropdownMenuLabel>Filter by</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              onClick={() => setTaskFilter("All")}
            >
              All
            </DropdownMenuItem>
            <DropdownMenuItem
              onClick={() => setTaskFilter("Running")}
            >
              Running
            </DropdownMenuItem>
            <DropdownMenuItem
              onClick={() => setTaskFilter("Completed")}
            >
              Completed
            </DropdownMenuItem>
            <DropdownMenuItem
              onClick={() => setTaskFilter("Failed")}
            >
              Failed
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      {currentTaskTitle ? (
        <div className="mt-3 px-2">
          <button
            onClick={() => {
              onViewChange?.("new");
            }}
            className={cn(
              "flex w-full items-center gap-3 rounded-md px-3 py-2 text-left text-[13px] transition-colors",
              activeView === "task"
                ? "bg-sidebar-accent text-sidebar-foreground"
                : "text-sidebar-foreground/85 hover:bg-sidebar-accent",
            )}
          >
            <LoaderCircle className="size-4 animate-spin text-amber-400" />
            <span className="min-w-0 flex-1 truncate">{currentTaskTitle}</span>
          </button>
        </div>
      ) : (
        <div className="flex flex-1 flex-col items-center justify-center px-6 text-center">
          <MessageSquareDashed className="size-9 text-muted-foreground/60" strokeDasharray="3 3" />
          <p className="mt-3 text-xs text-muted-foreground">
            Create a new task to get started
          </p>
        </div>
      )}

      {/* Footer */}
      <div className="flex items-center px-4 py-3">
        <button
          className="text-muted-foreground hover:text-sidebar-foreground"
          aria-label="Settings"
          onClick={onSettings}
        >
          <Settings className="size-4" />
        </button>
      </div>
    </aside>
  );
}
