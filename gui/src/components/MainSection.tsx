import { useState } from "react";
import {
  Plus,
  ArrowUp,
  Check,
  ChevronDown,
  RefreshCw,
  Shield,
  ShieldCheck,
  ShieldQuestion,
  FolderGit2,
  Monitor,
  GitBranch,
  ShoppingBag,
  Code2,
  Laptop,
  PenTool,
  MoreHorizontal,
  Film,
  Smartphone,
  Table2,
  BarChart3,
  AudioLines,
  MessageSquareText,
  BookOpen,
  ExternalLink,
  Settings,
  Paperclip,
  Puzzle,
  Copy,
  Search,
  CalendarDays,
  ArrowRight,
  Workflow,
  Inbox,
  Archive,
  Star,
  Grid2X2,
  List,
  SlidersHorizontal,
  Edit3,
} from "lucide-react";
import { sendOnyxMessage } from "@/lib/onyx";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

const quickActions = [
  { label: "Create slides", prompt: "Create a polished slide outline for my project.", icon: ShoppingBag },
  { label: "Build website", prompt: "Build a website plan with pages, sections, and copy.", icon: Code2 },
  { label: "Develop desktop apps", prompt: "Plan a desktop app feature set and implementation steps.", icon: Laptop },
  { label: "Design", prompt: "Design a clean interface direction for this app.", icon: PenTool },
  { label: "More", prompt: "", icon: MoreHorizontal },
];

const moreActions = [
  { label: "Video", prompt: "Create a video plan with scenes, script, and production notes for ", icon: Film },
  { label: "Develop apps", prompt: "Plan an app with screens, features, data model, and build steps for ", icon: Smartphone },
  { label: "Schedule tasks", prompt: "Create a scheduled task workflow for ", icon: CalendarDays },
  { label: "Wide Research", prompt: "Run wide research and organize findings about ", icon: Search },
  { label: "Spreadsheet", prompt: "Create a spreadsheet structure with columns, formulas, and summaries for ", icon: Table2 },
  { label: "Visualization", prompt: "Create a visualization plan with charts and insights for ", icon: BarChart3 },
  { label: "Audio", prompt: "Create an audio script or production plan for ", icon: AudioLines },
  { label: "Chat mode", prompt: "Switch into chat mode and help me explore ", icon: MessageSquareText },
  { label: "Playbook", prompt: "Create a step-by-step playbook for ", icon: BookOpen, external: true },
];

const skills = [
  {
    name: "skill-creator",
    prompt: "Use the skill-creator workflow to help me create or update a Codex skill.",
  },
  {
    name: "imagegen",
    prompt: "Use image generation to create a polished visual asset for ",
  },
  {
    name: "documents",
    prompt: "Help me create or refine a document for ",
  },
];

type ChatMessage = {
  id: number;
  role: "user" | "assistant";
  text: string;
};

type MainSectionProps = {
  activeView?: string;
  scheduledItems?: { id: string; title: string; time: string }[];
  onNewTask?: () => void;
  onTaskCreated?: (title: string) => void;
  onScheduleTask?: (title: string, time: string) => void;
};

function taskTitleFromPrompt(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return "New Onyx task";
  return trimmed.length > 34 ? `${trimmed.slice(0, 31)}...` : trimmed;
}

function renderInlineMarkdown(text: string) {
  return text.split(/(\*\*[^*]+\*\*|`[^`]+`)/g).map((part, index) => {
    if (part.startsWith("**") && part.endsWith("**")) {
      return <strong key={index}>{part.slice(2, -2)}</strong>;
    }
    if (part.startsWith("`") && part.endsWith("`")) {
      return (
        <code key={index} className="rounded bg-accent px-1 py-0.5 text-[0.9em] text-foreground">
          {part.slice(1, -1)}
        </code>
      );
    }
    return part;
  });
}

function FormattedResponse({ text }: { text: string }) {
  const lines = text.split(/\r?\n/);
  const blocks = [];

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index].trim();
    if (!line) continue;
    if (/^-{3,}$/.test(line)) {
      blocks.push(<hr key={`hr-${index}`} className="my-4 border-border" />);
      continue;
    }
    if (line.startsWith("### ")) {
      blocks.push(
        <h4 key={`h3-${index}`} className="mt-4 text-base font-semibold">
          {renderInlineMarkdown(line.slice(4))}
        </h4>,
      );
      continue;
    }
    if (line.startsWith("## ")) {
      blocks.push(
        <h3 key={`h2-${index}`} className="mt-5 text-lg font-semibold">
          {renderInlineMarkdown(line.slice(3))}
        </h3>,
      );
      continue;
    }
    if (line.startsWith("# ")) {
      blocks.push(
        <h2 key={`h1-${index}`} className="font-serif text-2xl font-semibold tracking-tight">
          {renderInlineMarkdown(line.slice(2))}
        </h2>,
      );
      continue;
    }
    if (/^\d+\.\s+/.test(line)) {
      const items = [];
      while (index < lines.length && /^\d+\.\s+/.test(lines[index].trim())) {
        items.push(lines[index].trim().replace(/^\d+\.\s+/, ""));
        index += 1;
      }
      index -= 1;
      blocks.push(
        <ol key={`ol-${index}`} className="my-3 list-decimal space-y-2 pl-6">
          {items.map((item, itemIndex) => (
            <li key={itemIndex}>{renderInlineMarkdown(item)}</li>
          ))}
        </ol>,
      );
      continue;
    }
    if (/^[-*]\s+/.test(line)) {
      const items = [];
      while (index < lines.length && /^[-*]\s+/.test(lines[index].trim())) {
        items.push(lines[index].trim().replace(/^[-*]\s+/, ""));
        index += 1;
      }
      index -= 1;
      blocks.push(
        <ul key={`ul-${index}`} className="my-3 list-disc space-y-2 pl-6">
          {items.map((item, itemIndex) => (
            <li key={itemIndex}>{renderInlineMarkdown(item)}</li>
          ))}
        </ul>,
      );
      continue;
    }

    const paragraph = [line];
    while (
      index + 1 < lines.length &&
      lines[index + 1].trim() &&
      !/^(#{1,3}\s+|\d+\.\s+|[-*]\s+|-{3,}$)/.test(lines[index + 1].trim())
    ) {
      index += 1;
      paragraph.push(lines[index].trim());
    }
    blocks.push(
      <p key={`p-${index}`} className="my-3">
        {renderInlineMarkdown(paragraph.join(" "))}
      </p>,
    );
  }

  return <div className="space-y-1">{blocks}</div>;
}

export function MainSection({
  activeView = "new",
  scheduledItems = [],
  onNewTask,
  onTaskCreated,
  onScheduleTask,
}: MainSectionProps) {
  const [prompt, setPrompt] = useState("");
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isSending, setIsSending] = useState(false);
  const [accessMode, setAccessMode] = useState("Full access");
  const [projectName, setProjectName] = useState("Onyx_Brain_Project");
  const [workMode, setWorkMode] = useState("Work locally");
  const [branchName, setBranchName] = useState("master");

  const openLocalFilePicker = () => {
    const input = document.createElement("input");
    input.type = "file";
    input.multiple = true;
    input.onchange = () => {
      const files = Array.from(input.files ?? []).map((file) => file.name);
      if (files.length > 0) {
        setPrompt((current) =>
          `${current}${current ? "\n" : ""}Attached files: ${files.join(", ")}`,
        );
      }
    };
    input.click();
  };

  const selectAccessMode = (mode: string) => {
    setAccessMode(mode);
  };

  const conversationText = () =>
    messages
      .map((message) => `${message.role === "user" ? "You" : "Onyx"}:\n${message.text}`)
      .join("\n\n");

  const copyConversation = async () => {
    try {
      await navigator.clipboard.writeText(conversationText());
    } catch {
      await navigator.clipboard?.writeText?.("");
    }
  };

  const saveTranscript = () => {
    const blob = new Blob([conversationText()], { type: "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = "onyx-transcript.txt";
    link.click();
    URL.revokeObjectURL(url);
  };

  const renameTask = () => {
    const nextTitle = window.prompt("Rename task", taskTitleFromPrompt(messages[0]?.text || ""));
    if (!nextTitle?.trim()) return;
    onTaskCreated?.(taskTitleFromPrompt(nextTitle));
  };

  const archiveTask = () => {
    setMessages([]);
    onNewTask?.();
  };

  const send = async () => {
    const text = prompt.trim();
    if (!text || isSending) {
      return;
    }
    const createdAt = Date.now();
    setMessages((current) => [
      ...current,
      { id: createdAt, role: "user", text },
      { id: createdAt + 1, role: "assistant", text: "Onyx is thinking..." },
    ]);
    onTaskCreated?.(taskTitleFromPrompt(text));
    setPrompt("");
    setIsSending(true);
    try {
      const response = await sendOnyxMessage(text);
      setMessages((current) =>
        current.map((message) =>
          message.id === createdAt + 1 ? { ...message, text: response } : message,
        ),
      );
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unknown error";
      setMessages((current) =>
        current.map((item) =>
          item.id === createdAt + 1
            ? { ...item, text: `Onyx could not run in the background yet.\n\n${message}` }
            : item,
        ),
      );
    } finally {
      setIsSending(false);
    }
  };

  if (activeView === "scheduled") {
    return (
      <ScheduledView
        items={scheduledItems}
        onScheduleTask={onScheduleTask}
      />
    );
  }

  if (activeView === "library") {
    return <LibraryView onNewTask={onNewTask} />;
  }

  if (messages.length > 0) {
    return (
      <main className="relative flex h-screen flex-1 flex-col bg-background">
        <header className="flex h-14 items-center justify-between px-6">
          <div
            className="px-2 py-1 text-sm font-semibold"
          >
            Onyx 1.0 Lite
          </div>
          <div className="flex items-center gap-2">
            <button
              aria-label="Copy conversation"
              onClick={copyConversation}
              className="grid size-8 place-items-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
            >
              <Copy className="size-4" />
            </button>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button
                  aria-label="More actions"
                  className="grid size-8 place-items-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
                >
                  <MoreHorizontal className="size-4" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-48">
                <DropdownMenuItem onClick={copyConversation}>
                  <Copy className="mr-2 size-4" /> Copy conversation
                </DropdownMenuItem>
                <DropdownMenuItem onClick={saveTranscript}>
                  Save transcript
                </DropdownMenuItem>
                <DropdownMenuItem onClick={renameTask}>
                  Rename task
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem onClick={archiveTask}>
                  Archive task
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </header>

        <div className="flex-1 overflow-auto px-6 pb-36 pt-8">
          <div className="mx-auto flex w-full max-w-3xl flex-col gap-8">
            {messages.map((message) =>
              message.role === "user" ? (
                <div key={message.id} className="flex justify-end">
                  <div className="max-w-[52%] truncate rounded-xl bg-card px-4 py-3 text-sm text-foreground">
                    {message.text}
                  </div>
                </div>
              ) : (
                <div key={message.id} className="max-w-3xl text-[15px] leading-7 text-foreground">
                  <div className="mb-3 flex items-center gap-2">
                    <span className="font-serif text-xl font-semibold">Onyx</span>
                    <span className="rounded-md border border-border px-1.5 py-0.5 text-[11px] text-muted-foreground">
                      Lite
                    </span>
                  </div>
                  <FormattedResponse text={message.text} />
                  <button
                    onClick={() => setPrompt("Continue this task from where you left off.")}
                    className="mt-4 rounded-lg bg-foreground px-3 py-2 text-sm font-medium text-background hover:opacity-90"
                  >
                    Continue <span className="text-background/70">55s</span>
                  </button>
                </div>
              ),
            )}
          </div>
        </div>

        <div className="absolute bottom-4 left-1/2 w-full max-w-3xl -translate-x-1/2 px-4">
          <div className="rounded-[22px] border border-border bg-card px-3 py-3 shadow-[0_12px_32px_rgba(0,0,0,0.26)]">
            <textarea
              rows={1}
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) {
                  e.preventDefault();
                  send();
                }
              }}
              placeholder="Send message to Onyx"
              className="min-h-10 w-full resize-none bg-transparent px-1 text-[15px] leading-6 text-foreground placeholder:text-muted-foreground focus:outline-none"
            />
            <div className="mt-2 flex items-center gap-2">
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <button
                    aria-label="Add attachment"
                    className="grid size-8 place-items-center rounded-full border border-border text-muted-foreground hover:bg-accent hover:text-foreground"
                  >
                    <Plus className="size-4" />
                  </button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" className="w-56">
                  <DropdownMenuItem onClick={openLocalFilePicker}>
                    <Paperclip className="mr-2 size-4" /> Add from local files
                  </DropdownMenuItem>
                  <DropdownMenuSub>
                    <DropdownMenuSubTrigger>
                      <Puzzle className="mr-2 size-4" /> Use Skills
                    </DropdownMenuSubTrigger>
                    <DropdownMenuSubContent className="w-72">
                      <DropdownMenuLabel>Search Skills</DropdownMenuLabel>
                      <DropdownMenuSeparator />
                      {skills.map((skill) => (
                        <DropdownMenuItem
                          key={skill.name}
                          onClick={() => {
                            setPrompt(skill.prompt);
                          }}
                        >
                          <Puzzle className="mr-2 size-4" />
                          <span className="truncate">{skill.name}</span>
                        </DropdownMenuItem>
                      ))}
                    </DropdownMenuSubContent>
                  </DropdownMenuSub>
                </DropdownMenuContent>
              </DropdownMenu>
              <button
                aria-label="Send message"
                onClick={send}
                disabled={!prompt.trim() || isSending}
                className="ml-auto grid size-8 place-items-center rounded-full bg-foreground text-background transition-opacity hover:opacity-90 disabled:cursor-not-allowed disabled:bg-accent disabled:text-muted-foreground disabled:opacity-100"
              >
                <ArrowUp className="size-4" />
              </button>
            </div>
          </div>
        </div>
      </main>
    );
  }

  return (
    <main className="relative flex h-screen flex-1 flex-col bg-background">
      {/* Center stack */}
      <div className="flex flex-1 flex-col items-center justify-center px-6">
        <div className="w-full max-w-[730px]">
          {/* Heading */}
          <h1 className="text-center font-serif text-5xl tracking-tight text-foreground/95">
            Where should we start?
          </h1>

          {/* Composer */}
          <div className="mt-10">
            <div className="relative z-10 flex w-full flex-col rounded-[18px] border border-border bg-card/95 px-3 py-3 shadow-[0_12px_32px_rgba(0,0,0,0.18)] transition-colors focus-within:border-foreground/25">
              <textarea
                rows={1}
                value={prompt}
                onChange={(e) => setPrompt(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && !e.shiftKey) {
                    e.preventDefault();
                    send();
                  }
                }}
                placeholder="Ask Onyx anything. @ to use plugins or mention files"
                className="min-h-[38px] max-h-[120px] w-full resize-none overflow-auto border-0 bg-transparent px-1 py-0.5 text-[14px] leading-6 text-foreground placeholder:text-muted-foreground focus:outline-none"
              />
              <div className="mt-1 flex items-center gap-2">
                <div className="flex shrink-0 items-center gap-3">
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <button
                        aria-label="Add attachment"
                        className="grid size-8 place-items-center rounded-full text-muted-foreground hover:bg-accent hover:text-foreground"
                      >
                        <Plus className="size-5" />
                      </button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start" className="w-56">
                      <DropdownMenuItem onClick={openLocalFilePicker}>
                        <Paperclip className="mr-2 size-4" /> Add from local files
                      </DropdownMenuItem>
                      <DropdownMenuSub>
                        <DropdownMenuSubTrigger>
                          <Puzzle className="mr-2 size-4" /> Use Skills
                        </DropdownMenuSubTrigger>
                        <DropdownMenuSubContent className="w-72">
                          <DropdownMenuLabel>Search Skills</DropdownMenuLabel>
                          <DropdownMenuSeparator />
                          {skills.map((skill) => (
                            <DropdownMenuItem
                              key={skill.name}
                              onClick={() => {
                                setPrompt(skill.prompt);
                              }}
                            >
                              <Puzzle className="mr-2 size-4" />
                              <span className="truncate">{skill.name}</span>
                            </DropdownMenuItem>
                          ))}
                          <DropdownMenuSeparator />
                          <DropdownMenuItem onClick={() => setPrompt("Manage skills for this workspace.")}>
                            <Settings className="mr-2 size-4" /> Manage Skills
                          </DropdownMenuItem>
                        </DropdownMenuSubContent>
                      </DropdownMenuSub>
                    </DropdownMenuContent>
                  </DropdownMenu>

                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <button
                        aria-label="Access mode"
                        className="flex h-8 items-center gap-1.5 rounded-full px-2 text-sm font-medium text-orange-400 hover:bg-accent"
                      >
                        <Shield className="size-4" />
                        {accessMode}
                        <ChevronDown className="size-3.5" />
                      </button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start" className="w-44 rounded-lg border-border bg-card p-1">
                      <DropdownMenuItem
                        onSelect={() => selectAccessMode("Default permissions")}
                        className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-2 text-sm"
                      >
                        <ShieldQuestion className="size-4 text-muted-foreground" />
                        <span className="flex-1">Default permissions</span>
                        {accessMode === "Default permissions" && <Check className="size-4 text-muted-foreground" />}
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onSelect={() => selectAccessMode("Auto-review")}
                        className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-2 text-sm"
                      >
                        <Shield className="size-4 text-muted-foreground" />
                        <span className="flex-1">Auto-review</span>
                        {accessMode === "Auto-review" && <Check className="size-4 text-muted-foreground" />}
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onSelect={() => selectAccessMode("Full access")}
                        className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-2 text-sm"
                      >
                        <ShieldCheck className="size-4 text-muted-foreground" />
                        <span className="flex-1">Full access</span>
                        {accessMode === "Full access" && <Check className="size-4 text-muted-foreground" />}
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>

                <div className="ml-auto flex min-w-0 shrink items-center gap-2">
                  <button
                    aria-label="Send task"
                    onClick={send}
                    disabled={!prompt.trim() || isSending}
                    className="grid size-9 place-items-center rounded-full bg-foreground text-background transition-opacity hover:opacity-90 disabled:cursor-not-allowed disabled:bg-accent disabled:text-muted-foreground disabled:opacity-100"
                  >
                    <ArrowUp className="size-5" />
                  </button>
                </div>
              </div>
            </div>
            <div className="-mt-[8px] flex min-h-12 flex-wrap items-end gap-4 rounded-b-[22px] bg-card/45 px-4 pb-3 pt-4 text-[12.5px] text-muted-foreground">
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <button className="flex items-center gap-1.5 hover:text-foreground">
                      <FolderGit2 className="size-4" />
                      {projectName}
                      <ChevronDown className="size-3.5" />
                    </button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="start">
                    <DropdownMenuItem onClick={() => setProjectName("Onyx_Brain_Project")}>
                      Onyx_Brain_Project
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={() => setProjectName("No project")}>
                      No project
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <button className="flex items-center gap-1.5 hover:text-foreground">
                      <Monitor className="size-4" />
                      {workMode}
                      <ChevronDown className="size-3.5" />
                    </button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="start">
                    <DropdownMenuItem onClick={() => setWorkMode("Work locally")}>
                      Work locally
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={() => setWorkMode("Cloud workspace")}>
                      Cloud workspace
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <button className="flex items-center gap-1.5 hover:text-foreground">
                      <GitBranch className="size-4" />
                      {branchName}
                      <ChevronDown className="size-3.5" />
                    </button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="start">
                    <DropdownMenuItem onClick={() => setBranchName("master")}>
                      master
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={() => setBranchName("develop")}>
                      develop
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
            </div>
          </div>

          {/* Quick actions */}
          <div className="mt-6 flex flex-wrap items-center justify-center gap-2">
            {quickActions.map(({ label, prompt: actionPrompt, icon: Icon }) =>
              label === "More" ? (
                <DropdownMenu key={label}>
                  <DropdownMenuTrigger asChild>
                    <button className="flex items-center gap-2 rounded-full border border-border bg-card/40 px-4 py-2 text-xs hover:bg-accent/40">
                      <Icon className="size-3.5 text-muted-foreground" />
                      {label}
                    </button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent
                    align="end"
                    sideOffset={8}
                    className="w-[200px] rounded-xl border-border bg-card p-2"
                  >
                    {moreActions.map(({ external, icon: MoreIcon, label: moreLabel, prompt: morePrompt }) => (
                      <DropdownMenuItem
                        key={moreLabel}
                        onClick={() => {
                          setPrompt(morePrompt);
                        }}
                        className="flex cursor-pointer items-center gap-3 rounded-lg px-2 py-2.5 text-sm"
                      >
                        <MoreIcon className="size-4 text-muted-foreground" />
                        <span className="min-w-0 flex-1">{moreLabel}</span>
                        {external && <ExternalLink className="size-3.5 text-muted-foreground" />}
                      </DropdownMenuItem>
                    ))}
                  </DropdownMenuContent>
                </DropdownMenu>
              ) : (
                <button
                  key={label}
                onClick={() => {
                  setPrompt(actionPrompt);
                }}
                  className="flex items-center gap-2 rounded-full border border-border bg-card/40 px-4 py-2 text-xs hover:bg-accent/40"
                >
                  <Icon className="size-3.5 text-muted-foreground" />
                  {label}
                </button>
              ),
            )}
          </div>
        </div>
      </div>
    </main>
  );
}

function ScheduledView({
  items,
  onScheduleTask,
}: {
  items: { id: string; title: string; time: string }[];
  onScheduleTask?: (title: string, time: string) => void;
}) {
  const examples = [
    {
      icon: RefreshCw,
      text: "Set up automated monitoring for any topic, competitor, or keyword.",
    },
    {
      icon: Inbox,
      text: "Get a daily summary of your inbox and schedule before starting your day.",
    },
    {
      icon: Workflow,
      text: "Turn any manual, multi-step process into an automated pipeline that runs on schedule.",
    },
  ];

  return (
    <main className="flex h-screen flex-1 flex-col bg-background p-5">
      <h1 className="text-base font-semibold">Scheduled</h1>
      <div className="mx-auto w-full max-w-4xl">
        {items?.length > 0 ? (
          <div className="space-y-4">
            {items.map((item) => (
              <div key={item.id} className="flex items-center gap-4 rounded-2xl border border-border bg-card p-4 shadow-sm">
                <div className="grid size-10 place-items-center rounded-full bg-primary/10 text-primary">
                  <CalendarDays className="size-5 text-primary" />
                </div>
                <div className="flex-1 min-w-0">
                  <h3 className="font-semibold text-foreground truncate">{item.title}</h3>
                  <p className="text-sm text-muted-foreground">{item.time}</p>
                </div>
                <button className="text-muted-foreground hover:text-foreground p-2">
                  <MoreHorizontal className="size-5" />
                </button>
              </div>
            ))}
            <button
              onClick={() =>
                onScheduleTask?.("Follow up on this project", "Tomorrow at 9:00 AM")
              }
              className="w-full flex items-center justify-center gap-2 rounded-2xl border border-dashed border-border p-4 text-sm text-muted-foreground hover:bg-accent/30 transition-colors"
            >
              <Plus className="size-4" /> Schedule another task
            </button>
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center py-10">
            <div className="mx-auto mb-14 grid size-28 place-items-center rounded-2xl border border-border bg-card shadow-[0_0_48px_rgba(255,255,255,0.05)]">
              <CalendarDays className="size-14 text-muted-foreground" />
            </div>
            <h2 className="font-serif text-3xl font-semibold tracking-tight text-center text-foreground">
              Onyx works independently, without you asking
            </h2>
            <div className="mt-7 space-y-3 w-full max-w-2xl">
              {examples.map(({ icon: Icon, text }) => (
                <button
                  key={text}
                  onClick={() => onScheduleTask?.(text, "Every weekday at 9:00 AM")}
                  className="flex w-full items-center gap-4 rounded-2xl border border-border px-4 py-4 text-left text-sm hover:bg-accent/50"
                >
                  <Icon className="size-4 text-muted-foreground" />
                  <span className="min-w-0 flex-1">{text}</span>
                  <ArrowRight className="size-4 text-muted-foreground" />
                </button>
              ))}
            </div>
            <button
              onClick={() =>
                onScheduleTask?.("Create a weekly workspace review", "Every Monday at 9:00 AM")
              }
              className="mt-6 inline-flex items-center gap-2 rounded-xl bg-foreground px-4 py-3 text-sm font-semibold text-background hover:opacity-90"
            >
              <Plus className="size-4" /> Create your scheduled task
            </button>
          </div>
        )}
      </div>
    </main>
  );
}

function LibraryView({ onNewTask }: { onNewTask?: () => void }) {
  const [filter, setFilter] = useState("All");
  const [viewMode, setViewMode] = useState<"grid" | "list">("grid");

  return (
    <main className="flex h-screen flex-1 flex-col bg-background p-5">
      <h1 className="text-base font-semibold">Library</h1>
      <div className="mt-5 flex items-center justify-between">
        <div className="ml-[18%] flex items-center gap-2">
          <button
            onClick={() => setFilter((current) => (current === "All" ? "Recent" : "All"))}
            className="flex items-center gap-2 rounded-lg border border-border px-3 py-2 text-sm hover:bg-accent"
          >
            <SlidersHorizontal className="size-4" /> {filter}
          </button>
          <button
            onClick={() => setFilter("My favorites")}
            className="flex items-center gap-2 rounded-lg border border-border px-3 py-2 text-sm hover:bg-accent"
          >
            <Star className="size-4" /> My favorites
          </button>
        </div>
        <div className="mr-[18%] flex items-center gap-2">
          <div className="flex h-9 w-48 items-center gap-2 rounded-lg border border-border px-3">
            <Search className="size-4 text-muted-foreground" />
            <input
              placeholder="Search files"
              className="min-w-0 flex-1 bg-transparent text-sm focus:outline-none"
            />
          </div>
          <button
            aria-label="Grid view"
            onClick={() => setViewMode("grid")}
            className={`grid size-9 place-items-center rounded-lg border border-border hover:bg-accent ${
              viewMode === "grid" ? "bg-accent text-foreground" : ""
            }`}
          >
            <Grid2X2 className="size-4" />
          </button>
          <button
            aria-label="List view"
            onClick={() => setViewMode("list")}
            className={`grid size-9 place-items-center rounded-lg border border-border hover:bg-accent ${
              viewMode === "list" ? "bg-accent text-foreground" : ""
            }`}
          >
            <List className="size-4" />
          </button>
        </div>
      </div>
      <div className="flex flex-1 items-center justify-center">
        <div className="text-center">
          <Archive className="mx-auto size-10 text-muted-foreground" />
          <h2 className="mt-5 text-base font-semibold">Nothing in the library</h2>
          <p className="mt-2 text-sm text-muted-foreground">
            Build your own knowledge base by creating new tasks.
          </p>
          <button
            onClick={onNewTask}
            className="mt-6 inline-flex items-center gap-2 rounded-lg bg-foreground px-4 py-3 text-sm font-semibold text-background hover:opacity-90"
          >
            <Edit3 className="size-4" /> New task
          </button>
        </div>
      </div>
    </main>
  );
}
