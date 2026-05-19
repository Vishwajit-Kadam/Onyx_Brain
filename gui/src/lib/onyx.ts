type TauriCore = {
  invoke: <T>(command: string, args?: Record<string, unknown>) => Promise<T>;
};

type ChatResponse = {
  response: string;
};

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
    __TAURI__?: {
      core?: TauriCore;
    };
  }
}

async function sendViaTauri(input: string) {
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    throw new Error("Tauri command bridge is not available");
  }
  const result = await invoke<ChatResponse>("send_chat_message", { input });
  return result.response;
}

async function sendViaDevServer(input: string) {
  const response = await fetch("http://127.0.0.1:5174/api/chat", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ input }),
  });
  const result = (await response.json()) as Partial<ChatResponse> & { error?: string };
  if (!response.ok || !result.response) {
    throw new Error(result.error || "Onyx did not return a response");
  }
  return result.response;
}

export async function sendOnyxMessage(input: string) {
  if (window.__TAURI_INTERNALS__) {
    return sendViaTauri(input);
  }
  return sendViaDevServer(input);
}
