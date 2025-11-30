<script lang="ts">
  import { Textarea } from "$lib/components/ui/textarea";
  import { Spinner } from "$lib/components/ui/spinner";
  import { Button } from "$lib/components/ui/button";
  import { tick, onMount, onDestroy } from "svelte";
  import { marked } from "marked";
  import {
    getCurrentWindow,
    currentMonitor,
    LogicalSize,
  } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";

  interface Message {
    role: "user" | "assistant";
    content: string;
  }

  interface Settings {
    apiKey: string;
    selectedModel: string;
    darkMode: boolean;
    autoStart: boolean;
    systemPrompt: string;
  }

  let inputValue = $state("");
  let messages = $state<Message[]>([]);
  let isLoading = $state(false);
  let messagesContainer: HTMLDivElement | null = $state(null);
  let hasResized = $state(false);
  let apiKey = $state("");
  let selectedModel = $state("openai/gpt-oss-120b");
  let systemPrompt = $state("Keep your responses as concise, precise, to the point.\nAnswer the question in as few words as possible.\nNo Yapping.");
  let unlistenNewChat: UnlistenFn | null = null;
  let textareaRef: HTMLTextAreaElement | null = $state(null);
  let unlistenWindowFocus: UnlistenFn | null = null;

  // Configure marked options
  marked.setOptions({
    breaks: true,
    gfm: true,
  });

  function renderMarkdown(content: string): string {
    return marked.parse(content) as string;
  }

  function startNewChat() {
    messages = [];
    inputValue = "";
    hasResized = false;
    // Reset window size to initial
    resetWindowSize();
    // Focus the textarea after starting new chat
    tick().then(() => textareaRef?.focus());
  }

  function focusTextarea() {
    textareaRef?.focus();
  }

  async function resetWindowSize() {
    try {
      const window = getCurrentWindow();
      await window.setSize(new LogicalSize(500, 150));
    } catch (error) {
      console.error("Failed to reset window size:", error);
    }
  }

  async function closeWindow() {
    try {
      const window = getCurrentWindow();
      await window.hide();
    } catch (error) {
      console.error("Failed to close window:", error);
    }
  }

  async function quitApp() {
    try {
      await invoke("quit_app");
    } catch (error) {
      console.error("Failed to quit app:", error);
    }
  }

  async function handleLocalKeydown(event: KeyboardEvent) {
    const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
    const modifier = isMac ? event.metaKey : event.ctrlKey;

    if (modifier) {
      if (event.key === "w") {
        event.preventDefault();
        await closeWindow();
      } else if (event.key === "n") {
        event.preventDefault();
        startNewChat();
      } else if (event.key === "q") {
        event.preventDefault();
        await quitApp();
      }
    }
  }

  onMount(async () => {
    try {
      const settings = await invoke<Settings>("load_settings");
      apiKey = settings.apiKey;
      selectedModel = settings.selectedModel;
      systemPrompt = settings.systemPrompt || "";
    } catch (error) {
      console.error("Failed to load settings:", error);
    }

    // Listen for new-chat event from global shortcuts / tray
    unlistenNewChat = await listen("new-chat", () => {
      startNewChat();
    });

    // Listen for window focus events to refocus textarea
    const appWindow = getCurrentWindow();
    unlistenWindowFocus = await appWindow.onFocusChanged(({ payload: focused }) => {
      if (focused && !isLoading) {
        focusTextarea();
      }
    });

    // Add local keyboard shortcuts
    window.addEventListener("keydown", handleLocalKeydown);

    // Focus textarea on initial mount
    await tick();
    focusTextarea();
  });

  onDestroy(() => {
    if (unlistenNewChat) {
      unlistenNewChat();
    }
    if (unlistenWindowFocus) {
      unlistenWindowFocus();
    }
    window.removeEventListener("keydown", handleLocalKeydown);
  });

  const hasMessages = $derived(messages.length > 0);

  async function openSettings() {
    await invoke("open_settings");
  }

  async function resizeWindowTo70Percent() {
    if (hasResized) return;

    try {
      const monitor = await currentMonitor();
      if (monitor) {
        const screenHeight = monitor.size.height;
        const newHeight = Math.round(screenHeight * 0.7);
        const window = getCurrentWindow();
        const currentSize = await window.innerSize();
        await window.setSize(new LogicalSize(currentSize.width, newHeight));
        hasResized = true;
      }
    } catch (error) {
      console.error("Failed to resize window:", error);
    }
  }

  async function scrollToBottom() {
    await tick();
    if (messagesContainer) {
      messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }
  }

  async function sendMessage() {
    const rawContent = inputValue.trim();
    if (!rawContent || isLoading) return;

    // Check for web search command
    const useWebSearch = rawContent.startsWith("/s ");
    const content = useWebSearch ? rawContent.slice(3).trim() : rawContent;

    if (!content) return;

    // Resize window on first message
    if (!hasMessages) {
      await resizeWindowTo70Percent();
    }

    // Add user message
    messages.push({ role: "user", content });
    inputValue = "";
    isLoading = true;

    await scrollToBottom();

    try {
      if (!apiKey) {
        messages.push({
          role: "assistant",
          content: "Please set your API key in settings first.",
        });
        await scrollToBottom();
        isLoading = false;
        return;
      }

      // Build request body
      const requestBody: Record<string, unknown> = {
        model: useWebSearch ? `${selectedModel}:online` : selectedModel,
        provider: {
          sort: "throughput",
        },
        messages: [
          ...(systemPrompt ? [{ role: "system", content: systemPrompt }] : []),
          ...messages.map((m) => ({
            role: m.role,
            content: m.content,
          })),
        ],
      };

      // Add web search plugin if using /s command
      if (useWebSearch) {
        requestBody.plugins = [
          {
            id: "web",
            max_results: 5,
          },
        ];
      }

      const response = await fetch(
        "https://openrouter.ai/api/v1/chat/completions",
        {
          method: "POST",
          headers: {
            Authorization: `Bearer ${apiKey}`,
            "Content-Type": "application/json",
          },
          body: JSON.stringify(requestBody),
        }
      );

      if (!response.ok) {
        throw new Error(`API error: ${response.status}`);
      }

      const data = await response.json();
      const assistantMessage =
        data.choices[0]?.message?.content || "No response";

      messages.push({ role: "assistant", content: assistantMessage });
      await scrollToBottom();
    } catch (error) {
      console.error("Error:", error);
      messages.push({
        role: "assistant",
        content: "Sorry, there was an error processing your request.",
      });
      await scrollToBottom();
    } finally {
      isLoading = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && event.shiftKey) {
      event.preventDefault();
      sendMessage();
    }
  }

  let copiedIndex = $state<number | null>(null);

  async function copyMessage(content: string, index: number) {
    try {
      await navigator.clipboard.writeText(content);
      copiedIndex = index;
      setTimeout(() => {
        copiedIndex = null;
      }, 2000);
    } catch (error) {
      console.error("Failed to copy message:", error);
    }
  }
</script>

<main class="container" class:has-messages={hasMessages}>
  <Button
    variant="ghost"
    size="icon"
    class="settings-button"
    onclick={openSettings}
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="18"
      height="18"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path
        d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
      />
      <circle cx="12" cy="12" r="3" />
    </svg>
  </Button>

  {#if hasMessages}
    <div class="messages-area" bind:this={messagesContainer}>
      {#each messages as message, index}
        <div class="message {message.role}">
          {#if message.role === "assistant"}
            <div class="message-content prose prose-sm dark:prose-invert max-w-none">
              {@html renderMarkdown(message.content)}
            </div>
          {:else}
            <div class="message-content">
              {message.content}
            </div>
          {/if}
          <button
            class="copy-button"
            onclick={() => copyMessage(message.content, index)}
            title="Copy message"
          >
            {#if copiedIndex === index}
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <polyline points="20 6 9 17 4 12" />
              </svg>
            {:else}
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
                <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
              </svg>
            {/if}
          </button>
        </div>
      {/each}
      {#if isLoading}
        <div class="message assistant">
          <div class="message-content loading">
            <Spinner class="w-5 h-5" />
            <span>Thinking...</span>
          </div>
        </div>
      {/if}
    </div>
  {/if}

  <div class="input-area" class:centered={!hasMessages}>
    <div class="input-wrapper">
      <Textarea
        placeholder="Ask me anything..."
        bind:value={inputValue}
        bind:ref={textareaRef}
        onkeydown={handleKeydown}
        class="chat-input"
        disabled={isLoading}
      />
    </div>
  </div>
</main>

<style>
  .container {
    margin: 0;
    padding: 16px;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    background: transparent;
    position: relative;
  }

  :global(.settings-button) {
    position: absolute;
    top: 8px;
    right: 8px;
    opacity: 0.6;
    transition: opacity 0.2s ease;
  }

  :global(.settings-button:hover) {
    opacity: 1;
  }

  .container.has-messages {
    justify-content: flex-end;
  }

  .messages-area {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding-bottom: 16px;
    margin-bottom: 16px;
  }

  .message.user {
    align-self: flex-end;
  }

  .message.assistant {
    align-self: flex-start;
  }

  .message-content {
    padding: 12px 16px;
    border-radius: 12px;
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  .message.user .message-content {
    background: var(--primary);
    color: var(--primary-foreground);
  }

  .message.assistant .message-content {
    background: var(--card);
    color: var(--card-foreground);
    border: 1px solid var(--border);
  }

  /* Markdown prose styling overrides */
  .message.assistant .message-content :global(p) {
    margin: 0.5em 0;
  }

  .message.assistant .message-content :global(p:first-child) {
    margin-top: 0;
  }

  .message.assistant .message-content :global(p:last-child) {
    margin-bottom: 0;
  }

  .message.assistant .message-content :global(pre) {
    background: var(--muted);
    border-radius: 8px;
    padding: 12px;
    overflow-x: auto;
    margin: 0.5em 0;
  }

  .message.assistant .message-content :global(code) {
    background: var(--muted);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.875em;
  }

  .message.assistant .message-content :global(pre code) {
    background: transparent;
    padding: 0;
  }

  .message.assistant .message-content :global(ul) {
    margin: 0.5em 0;
    padding-left: 1.5em;
    list-style-type: disc;
  }

  .message.assistant .message-content :global(ol) {
    margin: 0.5em 0;
    padding-left: 1.5em;
    list-style-type: decimal;
  }

  .message.assistant .message-content :global(ul ul) {
    list-style-type: circle;
  }

  .message.assistant .message-content :global(ul ul ul) {
    list-style-type: square;
  }

  .message.assistant .message-content :global(li) {
    margin: 0.25em 0;
    display: list-item;
  }

  .message.assistant .message-content :global(a) {
    color: var(--primary);
    text-decoration: underline;
  }

  .message.assistant .message-content :global(blockquote) {
    border-left: 3px solid var(--border);
    padding-left: 1em;
    margin: 0.5em 0;
    color: var(--muted-foreground);
  }

  .message.assistant .message-content :global(h1),
  .message.assistant .message-content :global(h2),
  .message.assistant .message-content :global(h3),
  .message.assistant .message-content :global(h4) {
    margin: 0.75em 0 0.5em 0;
    font-weight: 600;
  }

  .message.assistant .message-content :global(table) {
    border-collapse: collapse;
    margin: 0.5em 0;
    width: 100%;
  }

  .message.assistant .message-content :global(th),
  .message.assistant .message-content :global(td) {
    border: 1px solid var(--border);
    padding: 8px;
    text-align: left;
  }

  .message.assistant .message-content :global(th) {
    background: var(--muted);
  }

  .message-content.loading {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .copy-button {
    position: absolute;
    top: 4px;
    right: 4px;
    padding: 4px;
    background: var(--muted);
    border: 1px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.2s ease;
    color: var(--muted-foreground);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .copy-button:hover {
    background: var(--accent);
    color: var(--accent-foreground);
  }

  .message:hover .copy-button {
    opacity: 1;
  }

  .message {
    display: flex;
    max-width: 100%;
    position: relative;
  }

  .input-area {
    width: 100%;
    transition: all 0.3s ease;
  }

  .input-area.centered {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .input-wrapper {
    display: flex;
    width: 100%;
    max-width: 800px;
  }

  .input-area.centered .input-wrapper {
    width: 100%;
  }

  :global(.chat-input) {
    flex: 1;
    min-height: 48px;
    max-height: 300px;
    resize: vertical;
    border-radius: 12px;
    background: var(--card);
    border: 1px solid var(--border);
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    color: var(--foreground);
    padding: 12px 16px;
  }
</style>
