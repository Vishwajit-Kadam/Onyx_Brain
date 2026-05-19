import { createFileRoute } from "@tanstack/react-router";
import type React from "react";
import { useEffect, useState } from "react";
import { AppSidebar } from "@/components/AppSidebar";
import { MainSection } from "@/components/MainSection";
import {
  Cable,
  ChevronRight,
  Clock,
  CircleHelp,
  FileText,
  Folder,
  HelpCircle,
  Moon,
  PanelLeft,
  Plus,
  Puzzle,
  Search,
  Sliders,
  Sparkles,
  Sun,
  X,
} from "lucide-react";

type Project = { id: string; name: string; threads: string[] };
type ScheduledItem = { id: string; title: string; time: string };
type TaskItem = { id: string; title: string; description: string };

export const Route = createFileRoute("/")({
  component: Index,
  head: () => ({
    meta: [
      { title: "Onyx - AI workspace for tasks, projects, search, and schedules" },
      {
        name: "description",
        content:
          "Onyx helps customers search tasks, organize projects, create threads, schedule work, and find important workspace results quickly.",
      },
      {
        name: "keywords",
        content:
          "Onyx, AI workspace, task search, project management, scheduled tasks, productivity assistant",
      },
      { property: "og:title", content: "Onyx - AI workspace for focused work" },
      {
        property: "og:description",
        content:
          "Search tasks, projects, threads, and scheduled work from one clean Onyx workspace.",
      },
    ],
  }),
});

function Index() {
  const [open, setOpen] = useState(true);
  const [activeView, setActiveView] = useState("new");
  const [currentTaskTitle, setCurrentTaskTitle] = useState<string | null>(null);
  const [taskResetKey, setTaskResetKey] = useState(0);
  const [showSearch, setShowSearch] = useState(false);
  const [showCreateProject, setShowCreateProject] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [projects, setProjects] = useState<Project[]>([]);
  const [tasks, setTasks] = useState<TaskItem[]>([]);
  const [scheduledItems, setScheduledItems] = useState<ScheduledItem[]>([]);

  const createId = () => Math.random().toString(36).slice(2, 10);

  const rememberTask = (title: string, description = "Active Onyx task session") => {
    setCurrentTaskTitle(title);
    setTasks((current) => {
      const existing = current.find((task) => task.title === title);
      if (existing) {
        return current.map((task) =>
          task.id === existing.id ? { ...task, description } : task,
        );
      }
      return [{ id: createId(), title, description }, ...current];
    });
  };

  const startNewTask = () => {
    setActiveView("new");
    setCurrentTaskTitle(null);
    setTaskResetKey((key) => key + 1);
  };

  useEffect(() => {
    document.title = "Onyx - What can I do for you?";
  }, []);

  return (
    <div className="flex min-h-screen w-full bg-background text-foreground">
      {open ? (
        <AppSidebar
          activeView={activeView}
          projects={projects}
          currentTaskTitle={currentTaskTitle}
          onCreateProject={() => setShowCreateProject(true)}
          onCreateThread={(projectId) => {
            setProjects((current) =>
              current.map((project) =>
                project.id === projectId
                  ? {
                      ...project,
                      threads: [...project.threads, `Thread ${project.threads.length + 1}`],
                    }
                  : project,
              ),
            );
          }}
          onNewTask={startNewTask}
          onSearch={() => setShowSearch(true)}
          onSettings={() => setShowSettings(true)}
          onToggle={() => setOpen(false)}
          onViewChange={setActiveView}
        />
      ) : (
        <button
          onClick={() => setOpen(true)}
          className="fixed left-3 top-4 z-20 grid size-8 place-items-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
        >
          <PanelLeft className="size-4" />
        </button>
      )}
      <MainSection
        key={taskResetKey}
        activeView={activeView}
        scheduledItems={scheduledItems}
        onNewTask={startNewTask}
        onTaskCreated={(title) => {
          rememberTask(title);
          setActiveView("new");
        }}
        onScheduleTask={(title, time) => {
          setScheduledItems((current) => [{ id: createId(), title, time }, ...current]);
          setActiveView("scheduled");
        }}
      />
      {showSearch && (
        <SearchModal
          currentTaskTitle={currentTaskTitle}
          tasks={tasks}
          projects={projects}
          scheduledItems={scheduledItems}
          onClose={() => setShowSearch(false)}
          onNewTask={() => {
            setShowSearch(false);
            startNewTask();
          }}
        />
      )}
      {showCreateProject && (
        <CreateProjectModal
          onClose={() => setShowCreateProject(false)}
          onCreate={(name) => {
            setProjects((current) => [...current, { id: createId(), name, threads: [] }]);
            setShowCreateProject(false);
          }}
        />
      )}
      {showSettings && <SettingsModal onClose={() => setShowSettings(false)} />}
    </div>
  );
}

function SettingsModal({ onClose }: { onClose: () => void }) {
  const [settingsPage, setSettingsPage] = useState("General");
  const [language, setLanguage] = useState("English");
  const [theme, setTheme] = useState("Auto");
  const menu = [
    { group: "Account", items: [
      { label: "General", icon: Sliders },
      { label: "Personalization", icon: Sparkles },
    ] },
    { group: "Features", items: [
      { label: "Skills", icon: Puzzle },
      { label: "Connectors", icon: Cable },
    ] },
  ];

  return (
    <div className="fixed inset-0 z-50 grid place-items-center bg-black/70 p-2">
      <div className="flex h-[92vh] w-full max-w-[1440px] overflow-hidden rounded-[22px] border border-border bg-background shadow-2xl">
        <aside className="w-56 shrink-0 border-r border-border bg-sidebar p-3">
          <div className="flex items-center gap-3 px-2 py-4">
            <div className="grid size-9 place-items-center rounded-full bg-gradient-to-br from-orange-400 via-pink-500 to-purple-600 text-xs font-bold">
              VK
            </div>
            <div className="min-w-0 flex-1">
              <div className="truncate text-sm font-semibold">Vishwajit Kadam</div>
              <div className="text-xs text-muted-foreground">Personal</div>
            </div>
          </div>
          <div className="my-2 h-px bg-border" />
          {menu.map((section) => (
            <div key={section.group} className="mt-4">
              <div className="px-2 pb-2 text-xs text-muted-foreground">{section.group}</div>
              <div className="space-y-1">
                {section.items.map(({ label, icon: Icon }) => {
                  const active = settingsPage === label;
                  return (
                  <button
                    key={label}
                    onClick={() => setSettingsPage(label)}
                    className={`flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm ${
                      active ? "bg-accent text-foreground" : "text-sidebar-foreground/85 hover:bg-sidebar-accent"
                    }`}
                  >
                    <Icon className="size-4" />
                    {label}
                  </button>
                  );
                })}
              </div>
            </div>
          ))}
          <div className="mt-5 border-t border-border pt-4">
            <button
              onClick={() => setSettingsPage("Help")}
              className="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm hover:bg-sidebar-accent"
            >
              <HelpCircle className="size-4" />
              Get help
            </button>
          </div>
        </aside>

        <main className="relative flex-1 px-16 py-10">
          <button
            aria-label="Close settings"
            onClick={onClose}
            className="absolute right-5 top-5 grid size-8 place-items-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
          >
            <X className="size-5" />
          </button>
          {settingsPage === "Personalization" && <PersonalizationSettings />}
          {(settingsPage === "Skills" || settingsPage === "Connectors" || settingsPage === "Help") && (
            <SimpleSettingsPage title={settingsPage} />
          )}
          <div className={settingsPage === "General" ? "max-w-3xl" : "hidden"}>
            <h2 className="text-2xl font-semibold">General</h2>
            <div className="mt-5 border-t border-border pt-7">
              <h3 className="text-base font-semibold">Appearance</h3>
              <label className="mt-5 block text-sm font-medium">Language</label>
              <button
                onClick={() => setLanguage((current) => (current === "English" ? "Hindi" : "English"))}
                className="mt-2 flex h-10 w-52 items-center justify-between rounded-lg bg-card px-3 text-sm hover:bg-accent"
              >
                {language} <span className="text-muted-foreground">?</span>
              </button>
              <div className="mt-7 text-sm font-medium">Theme</div>
              <div className="mt-3 grid w-[348px] grid-cols-3 gap-2">
                {[
                  { label: "Light", icon: Sun },
                  { label: "Dark", icon: Moon },
                  { label: "Auto", icon: Sliders },
                ].map(({ label, icon: Icon }) => (
                  <button
                    key={label}
                    onClick={() => setTheme(label)}
                    className={`grid h-16 place-items-center rounded-lg border text-sm ${
                      theme === label ? "border-foreground" : "border-border hover:bg-accent"
                    }`}
                  >
                    <Icon className="size-5 text-muted-foreground" />
                    <span>{label}</span>
                  </button>
                ))}
              </div>
            </div>
          </div>
        </main>
      </div>
    </div>
  );
}

function PersonalizationSettings() {
  const [tab, setTab] = useState("Profile");
  const [memoryImported, setMemoryImported] = useState(false);

  return (
    <div className="max-w-[768px]">
      <h2 className="text-2xl font-semibold">Personalization</h2>
      <p className="mt-1 text-sm text-muted-foreground">
        Manage who you are and what Onyx remembers
      </p>
      <div className="mt-5 border-t border-border" />

      <div className="mt-8 flex items-end gap-6 border-b border-border">
        <button
          onClick={() => setTab("Profile")}
          className={`pb-3 text-sm font-semibold ${
            tab === "Profile" ? "border-b-2 border-foreground text-foreground" : "text-muted-foreground"
          }`}
        >
          Profile
        </button>
        <button
          onClick={() => setTab("Knowledge")}
          className={`flex items-center gap-1.5 pb-3 text-sm font-semibold hover:text-foreground ${
            tab === "Knowledge" ? "border-b-2 border-foreground text-foreground" : "text-muted-foreground"
          }`}
        >
          Knowledge <CircleHelp className="size-3.5" />
        </button>
      </div>

      <button
        onClick={() => setMemoryImported(true)}
        className="mt-6 flex w-full items-center gap-4 rounded-xl border border-border bg-card px-4 py-4 text-left hover:bg-accent/40"
      >
        <span className="grid size-10 place-items-center rounded-lg bg-accent text-muted-foreground">
          <Sparkles className="size-5" />
        </span>
        <span className="min-w-0 flex-1">
          <span className="block text-sm font-semibold">
            {memoryImported ? "Memory import ready" : "Import memory from another AI"}
          </span>
          <span className="mt-1 block truncate text-sm text-muted-foreground">
            {memoryImported
              ? "Paste or attach exported conversations to fill your profile."
              : "Auto-fill your profile using conversations from other AI providers."}
          </span>
        </span>
        <ChevronRight className="size-4 text-muted-foreground" />
      </button>

      <div className="mt-7 grid grid-cols-2 gap-4">
        <label className="block">
          <span className="text-sm font-semibold">Nickname</span>
          <input
            placeholder="Jane"
            className="mt-2 h-9 w-full rounded-lg border border-transparent bg-card px-3 text-sm text-foreground placeholder:text-muted-foreground focus:border-foreground/25 focus:outline-none"
          />
        </label>
        <label className="block">
          <span className="text-sm font-semibold">Occupation</span>
          <input
            placeholder="Product designer"
            className="mt-2 h-9 w-full rounded-lg border border-transparent bg-card px-3 text-sm text-foreground placeholder:text-muted-foreground focus:border-foreground/25 focus:outline-none"
          />
        </label>
      </div>

      <label className="mt-5 block">
        <span className="text-sm font-semibold">More about you</span>
        <textarea
          placeholder="I'm an Analyst based in NYC. I work mainly in React and SQL..."
          rows={6}
          className="mt-2 w-full resize-none rounded-lg border border-transparent bg-card p-4 text-sm text-foreground placeholder:text-muted-foreground focus:border-foreground/25 focus:outline-none"
        />
      </label>
      <p className="mt-4 text-sm text-muted-foreground">
        Onyx uses this information to personalize responses across all tasks.
      </p>

      <div className="mt-5 border-t border-border pt-6">
        <label className="block">
          <span className="text-sm font-semibold">Custom Instructions</span>
          <textarea
            placeholder="Be concise and direct. Default to Python unless I say otherwise. When writing docs, use a professional tone and cite sources."
            rows={6}
            className="mt-2 w-full resize-none rounded-lg border border-transparent bg-card p-4 text-sm text-foreground placeholder:text-muted-foreground focus:border-foreground/25 focus:outline-none"
          />
        </label>
      </div>

      <button
        onClick={() => setMemoryImported(true)}
        className="mt-16 rounded-lg border border-border px-3 py-2 text-sm font-semibold hover:bg-accent"
      >
        {memoryImported ? "Memory import ready" : "Import memory"}
      </button>
    </div>
  );
}

function SimpleSettingsPage({ title }: { title: string }) {
  return (
    <div className="max-w-3xl">
      <h2 className="text-2xl font-semibold">{title}</h2>
      <div className="mt-5 border-t border-border pt-7">
        <p className="text-sm text-muted-foreground">
          {title === "Skills" &&
            "Choose and manage the skills Onyx can use in this workspace."}
          {title === "Connectors" &&
            "Connect local or cloud sources so Onyx can work with your project context."}
          {title === "Help" &&
            "Find setup guidance, support details, and workspace troubleshooting here."}
        </p>
      </div>
    </div>
  );
}

function SearchModal({
  currentTaskTitle,
  tasks,
  projects,
  scheduledItems,
  onClose,
  onNewTask,
}: {
  currentTaskTitle: string | null;
  tasks: TaskItem[];
  projects: Project[];
  scheduledItems: ScheduledItem[];
  onClose: () => void;
  onNewTask: () => void;
}) {
  const [searchQuery, setSearchQuery] = useState("");
  const query = searchQuery.trim().toLowerCase();
  const matches = (value: string) => !query || value.toLowerCase().includes(query);
  const taskResults = tasks.filter((task) => matches(`${task.title} ${task.description}`));
  const projectResults = projects.filter((project) =>
    matches(`${project.name} ${project.threads.join(" ")}`),
  );
  const threadResults = projects.flatMap((project) =>
    project.threads
      .filter((thread) => matches(`${project.name} ${thread}`))
      .map((thread) => ({ id: `${project.id}-${thread}`, projectName: project.name, title: thread })),
  );
  const scheduledResults = scheduledItems.filter((item) => matches(`${item.title} ${item.time}`));
  const hasResults =
    taskResults.length > 0 ||
    projectResults.length > 0 ||
    threadResults.length > 0 ||
    scheduledResults.length > 0;

  return (
    <div className="fixed inset-0 z-50 grid place-items-center bg-black/55 p-3">
      <div className="w-full max-w-3xl -translate-y-[3vh] overflow-hidden rounded-[22px] border border-border bg-card shadow-2xl">
        <div className="flex items-center gap-3 border-b border-border px-5 py-4">
          <Search className="size-5 text-muted-foreground" />
          <input
            autoFocus
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search tasks, projects, threads, schedules..."
            className="min-w-0 flex-1 bg-transparent text-base text-foreground placeholder:text-muted-foreground focus:outline-none"
          />
          <button
            aria-label="Close search"
            onClick={onClose}
            className="grid size-8 place-items-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
          >
            <X className="size-5" />
          </button>
        </div>
        <div className="max-h-[64vh] min-h-[420px] overflow-y-auto p-3">
          {!query && (
            <button
              onClick={onNewTask}
              className="flex w-full items-center gap-3 rounded-lg bg-accent px-3 py-3 text-left text-sm font-semibold hover:bg-accent/80"
            >
              <span className="grid size-8 place-items-center rounded-full bg-background text-foreground">
                <Plus className="size-4" />
              </span>
              New task
            </button>
          )}

          {taskResults.length > 0 && (
            <SearchSection title="Tasks">
              {taskResults.map((task) => (
                <SearchResult
                  key={task.id}
                  icon={<FileText className="size-4" />}
                  title={task.title}
                  detail={task.title === currentTaskTitle ? "Current task" : task.description}
                  onClick={onClose}
                />
              ))}
            </SearchSection>
          )}

          {projectResults.length > 0 && (
            <SearchSection title="Projects">
              {projectResults.map((project) => (
                <SearchResult
                  key={project.id}
                  icon={<Folder className="size-4" />}
                  title={project.name}
                  detail={
                    project.threads.length
                      ? `${project.threads.length} project thread${project.threads.length === 1 ? "" : "s"}`
                      : "Project workspace"
                  }
                  onClick={() => {
                    onClose();
                  }}
                />
              ))}
            </SearchSection>
          )}

          {threadResults.length > 0 && (
            <SearchSection title="Threads">
              {threadResults.map((thread) => (
                <SearchResult
                  key={thread.id}
                  icon={<Plus className="size-4" />}
                  title={thread.title}
                  detail={`Inside ${thread.projectName}`}
                  onClick={() => {
                    onClose();
                  }}
                />
              ))}
            </SearchSection>
          )}

          {scheduledResults.length > 0 && (
            <SearchSection title="Scheduled">
              {scheduledResults.map((item) => (
                <SearchResult
                  key={item.id}
                  icon={<Clock className="size-4" />}
                  title={item.title}
                  detail={item.time}
                  onClick={() => {
                    onClose();
                  }}
                />
              ))}
            </SearchSection>
          )}

          {!query && !hasResults && (
            <div className="mt-12 text-center text-sm text-muted-foreground">
              Create a task, project, thread, or schedule and it will appear here instantly.
            </div>
          )}

          {query && !hasResults && (
            <div className="mt-12 text-center">
              <p className="text-sm font-semibold">No matching workspace results</p>
              <p className="mt-2 text-sm text-muted-foreground">
                Try searching by task title, project name, thread number, or schedule time.
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function SearchSection({
  children,
  title,
}: {
  children: React.ReactNode;
  title: string;
}) {
  return (
    <section className="pt-3">
      <div className="px-3 text-xs font-medium uppercase tracking-wide text-muted-foreground">
        {title}
      </div>
      <div className="mt-2 space-y-1">{children}</div>
    </section>
  );
}

function SearchResult({
  detail,
  icon,
  onClick,
  title,
}: {
  detail: string;
  icon: React.ReactNode;
  onClick: () => void;
  title: string;
}) {
  return (
    <button
      onClick={onClick}
      className="flex w-full items-center gap-3 rounded-lg px-3 py-3 text-left hover:bg-accent/50"
    >
      <span className="grid size-8 place-items-center rounded-full bg-accent text-muted-foreground">
        {icon}
      </span>
      <span className="min-w-0 flex-1">
        <span className="block truncate text-sm font-semibold">{title}</span>
        <span className="block truncate text-xs text-muted-foreground">{detail}</span>
      </span>
    </button>
  );
}

function CreateProjectModal({
  onClose,
  onCreate,
}: {
  onClose: () => void;
  onCreate: (name: string) => void;
}) {
  const [name, setName] = useState("");
  const [connectorsOpen, setConnectorsOpen] = useState(false);
  return (
    <div className="fixed inset-0 z-50 grid place-items-center bg-black/65 p-4">
      <div className="w-full max-w-xl rounded-[22px] border border-border bg-card p-5 shadow-2xl">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold">Create project</h2>
          <button
            aria-label="Close create project"
            onClick={onClose}
            className="grid size-8 place-items-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
          >
            <X className="size-5" />
          </button>
        </div>
        <div className="mx-auto mt-6 grid size-16 place-items-center rounded-2xl bg-accent">
          <Folder className="size-8 text-muted-foreground" />
        </div>
        <label className="mt-6 block text-sm font-semibold">Project name</label>
        <input
          placeholder="Branding & Comms Project"
          value={name}
          onChange={(e) => setName(e.target.value)}
          className="mt-2 h-9 w-full rounded-md border border-border bg-transparent px-3 text-sm focus:outline-none focus:ring-1 focus:ring-foreground/25"
        />
        <label className="mt-5 block text-sm font-semibold">
          Instructions <span className="text-muted-foreground">(optional)</span>
        </label>
        <textarea
          rows={6}
          placeholder={"Keep responses concise and professional.\nUse our brand voice from the attached guidelines.\nAlways provide sources for important conclusions.\nNever publish without approval."}
          className="mt-2 w-full resize-none rounded-lg border border-transparent bg-background/60 p-3 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-foreground/25"
        />
        <button
          onClick={() => setConnectorsOpen((open) => !open)}
          className={`mt-4 flex h-10 w-full items-center justify-between rounded-lg border px-3 text-sm hover:bg-accent/50 ${
            connectorsOpen ? "border-foreground/40 bg-accent/30" : "border-border"
          }`}
        >
          <span>
            <span className="font-semibold">Connectors</span>{" "}
            <span className="text-muted-foreground">(optional)</span>
          </span>
          <span className="flex items-center gap-2 font-semibold">
            <Plus className="size-4" /> Add
          </span>
        </button>
        {connectorsOpen && (
          <div className="mt-2 rounded-lg border border-border bg-background/50 p-3 text-sm text-muted-foreground">
            Google Drive, OneDrive, and Figma connectors can be added from the project workspace.
          </div>
        )}
        <div className="mt-7 flex justify-end gap-2">
          <button onClick={onClose} className="rounded-lg border border-border px-4 py-2 text-sm font-semibold hover:bg-accent">
            Cancel
          </button>
          <button
            onClick={() => {
              if (name) {
                onCreate(name);
              }
            }}
            disabled={!name.trim()}
            className="rounded-lg bg-foreground px-4 py-2 text-sm font-semibold text-background hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-45"
          >
            Create
          </button>
        </div>
      </div>
    </div>
  );
}
