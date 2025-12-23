<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Switch } from "$lib/components/ui/switch";
  import { Separator } from "$lib/components/ui/separator";
  import { Textarea } from "$lib/components/ui/textarea";
  import * as Select from "$lib/components/ui/select";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  interface Settings {
    apiKey: string;
    selectedModel: string;
    darkMode: boolean;
    autoStart: boolean;
    systemPrompt: string;
    modelShortcuts: Record<string, string>;
    sendOnEnter: boolean;
  }

  interface ShortcutEntry {
    shortcut: string;
    model: string;
  }

  let apiKey = $state("");
  let selectedModel = $state("openai/gpt-oss-120b");
  let darkMode = $state(true);
  let autoStart = $state(false);
  let sendOnEnter = $state(false);
  let systemPrompt = $state(
    "Keep your responses as concise, precise, to the point.\nAnswer the question in as few words as possible.\nNo Yapping."
  );
  let modelShortcuts = $state<ShortcutEntry[]>([
    { shortcut: "h", model: "google/gemini-3-pro-preview" },
    { shortcut: "f", model: "google/gemini-3-flash-preview" },
    { shortcut: "o", model: "openai/gpt-oss-120b" },
  ]);
  let isLoading = $state(true);
  let saveMessage = $state("");
  let showSaveMessage = $state(false);

  const models = [
    { value: "openai/gpt-oss-120b", label: "GPT OSS 120B" },
    { value: "openai/gpt-oss-20b", label: "GPT OSS 20B" },
    {
      value: "google/gemini-3-flash-preview",
      label: "Gemini 2.5 Flash Preview 09-2025",
    },
    { value: "google/gemini-3-pro-preview", label: "Gemini 3 Pro Preview" },
  ];

  function getModelLabel(value: string): string {
    return models.find((m) => m.value === value)?.label ?? value;
  }

  function shortcutsToRecord(shortcuts: ShortcutEntry[]): Record<string, string> {
    const record: Record<string, string> = {};
    for (const entry of shortcuts) {
      if (entry.shortcut.trim()) {
        record[entry.shortcut.trim()] = entry.model;
      }
    }
    return record;
  }

  function recordToShortcuts(record: Record<string, string>): ShortcutEntry[] {
    return Object.entries(record).map(([shortcut, model]) => ({
      shortcut,
      model,
    }));
  }

  function addShortcut() {
    modelShortcuts = [...modelShortcuts, { shortcut: "", model: models[0].value }];
  }

  function removeShortcut(index: number) {
    modelShortcuts = modelShortcuts.filter((_, i) => i !== index);
  }

  onMount(async () => {
    try {
      const settings = await invoke<Settings>("load_settings");
      apiKey = settings.apiKey;
      selectedModel = settings.selectedModel;
      darkMode = settings.darkMode;
      autoStart = settings.autoStart;
      systemPrompt = settings.systemPrompt || "";
      if (settings.modelShortcuts && Object.keys(settings.modelShortcuts).length > 0) {
        modelShortcuts = recordToShortcuts(settings.modelShortcuts);
      }
      sendOnEnter = settings.sendOnEnter ?? false;
    } catch (error) {
      console.error("Failed to load settings:", error);
    } finally {
      isLoading = false;
    }
  });

  async function closeWindow() {
    const window = getCurrentWindow();
    await window.close();
  }

  async function handleCancel() {
    await closeWindow();
  }

  async function saveSettings() {
    try {
      await invoke("save_settings", {
        settings: {
          apiKey,
          selectedModel,
          darkMode,
          autoStart,
          sendOnEnter,
          systemPrompt,
          modelShortcuts: shortcutsToRecord(modelShortcuts),
        },
      });
      console.log("Settings saved:", {
        apiKey,
        selectedModel,
        darkMode,
        autoStart,
        sendOnEnter,
        systemPrompt,
        modelShortcuts: shortcutsToRecord(modelShortcuts),
      });
      saveMessage = "Settings saved successfully!";
      showSaveMessage = true;
      setTimeout(() => {
        showSaveMessage = false;
        closeWindow();
      }, 1500);
    } catch (error) {
      console.error("Failed to save settings:", error);
      saveMessage = "Failed to save settings";
      showSaveMessage = true;
      setTimeout(() => {
        showSaveMessage = false;
      }, 3000);
    }
  }
</script>

<div class="settings-container">
  {#if isLoading}
    <div class="loading">Loading settings...</div>
  {:else}
    <main class="settings-content">
      <section class="settings-section">
        <h2>API Configuration</h2>
        <div class="setting-item">
          <Label for="api-key">OpenRouter API Key</Label>
          <Input
            id="api-key"
            type="password"
            placeholder="sk-or-v1-..."
            bind:value={apiKey}
          />
        </div>
        <div class="setting-item">
          <Label for="model">Model</Label>
          <Select.Root type="single" bind:value={selectedModel}>
            <Select.Trigger class="w-full">
              {getModelLabel(selectedModel)}
            </Select.Trigger>
            <Select.Content>
              {#each models as model}
                <Select.Item value={model.value} label={model.label} />
              {/each}
            </Select.Content>
          </Select.Root>
        </div>
        <div class="setting-item">
          <Label for="system-prompt">System Prompt</Label>
          <Textarea
            id="system-prompt"
            placeholder="You are a helpful assistant..."
            bind:value={systemPrompt}
            class="system-prompt-input"
          />
        </div>
        <div class="setting-item">
          <div class="shortcuts-header">
            <Label>Model Shortcuts</Label>
            <Button variant="outline" size="sm" onclick={addShortcut}>+ Add</Button>
          </div>
          <p class="shortcuts-hint">Use /shortcut in chat to switch models (e.g., "/s /h" for web search with heavy model)</p>
          <div class="shortcuts-list">
            {#each modelShortcuts as entry, index}
              <div class="shortcut-row">
                <Input
                  type="text"
                  placeholder="shortcut"
                  bind:value={entry.shortcut}
                  class="shortcut-input"
                />
                <Select.Root type="single" bind:value={entry.model}>
                  <Select.Trigger class="shortcut-model-select">
                    {getModelLabel(entry.model)}
                  </Select.Trigger>
                  <Select.Content>
                    {#each models as model}
                      <Select.Item value={model.value} label={model.label} />
                    {/each}
                  </Select.Content>
                </Select.Root>
                <Button variant="ghost" size="icon" onclick={() => removeShortcut(index)} class="remove-shortcut-btn">
                  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                  </svg>
                </Button>
              </div>
            {/each}
          </div>
        </div>
      </section>

      <Separator />

      <section class="settings-section">
        <h2>Appearance</h2>
        <div class="setting-item row">
          <Label for="dark-mode">Dark Mode</Label>
          <Switch id="dark-mode" bind:checked={darkMode} />
        </div>
      </section>

      <Separator />

      <section class="settings-section">
        <h2>General</h2>
        <div class="setting-item row">
          <Label for="auto-start">Launch at startup</Label>
          <Switch id="auto-start" bind:checked={autoStart} />
        </div>
        <div class="setting-item row">
          <Label for="send-on-enter">Send message on Enter</Label>
          <Switch id="send-on-enter" bind:checked={sendOnEnter} />
        </div>
        <p class="setting-hint">
          When enabled, Enter sends your message and Shift+Enter creates a new line.
        </p>
      </section>
    </main>

    <footer class="settings-footer">
      <Button variant="outline" onclick={handleCancel}>Cancel</Button>
      <Button onclick={saveSettings}>Save</Button>
    </footer>

    {#if showSaveMessage}
      <div class="save-toast" class:error={saveMessage.includes("Failed")}>
        {saveMessage}
      </div>
    {/if}
  {/if}
</div>

<style>
  .settings-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--background);
    color: var(--foreground);
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--muted-foreground);
  }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }

  .settings-section {
    margin-bottom: 24px;
  }

  .settings-section h2 {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--muted-foreground);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 16px;
  }

  .setting-item {
    margin-bottom: 16px;
  }

  .setting-item.row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .setting-item :global(label) {
    display: block;
    margin-bottom: 8px;
  }

  .setting-item.row :global(label) {
    margin-bottom: 0;
  }

  :global(.system-prompt-input) {
    min-height: 100px;
    resize: vertical;
  }

  .shortcuts-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .shortcuts-hint {
    font-size: 0.75rem;
    color: var(--muted-foreground);
    margin-bottom: 12px;
  }

  .setting-hint {
    font-size: 0.75rem;
    color: var(--muted-foreground);
    margin: 4px 0 16px;
  }

  .shortcuts-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .shortcut-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  :global(.shortcut-input) {
    width: 80px;
    flex-shrink: 0;
  }

  :global(.shortcut-model-select) {
    flex: 1;
  }

  :global(.remove-shortcut-btn) {
    flex-shrink: 0;
    color: var(--muted-foreground);
  }

  :global(.remove-shortcut-btn:hover) {
    color: var(--destructive);
  }

  .settings-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 16px 20px;
    border-top: 1px solid var(--border);
  }

  .save-toast {
    position: fixed;
    bottom: 80px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--primary);
    color: var(--primary-foreground);
    padding: 12px 24px;
    border-radius: 8px;
    font-size: 0.875rem;
    font-weight: 500;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    animation: slideUp 0.3s ease-out;
    z-index: 100;
  }

  .save-toast.error {
    background: var(--destructive);
    color: var(--destructive-foreground);
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateX(-50%) translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateX(-50%) translateY(0);
    }
  }
</style>
