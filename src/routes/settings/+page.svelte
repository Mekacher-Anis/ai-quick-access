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
  }

  let apiKey = $state("");
  let selectedModel = $state("openai/gpt-oss-120b");
  let darkMode = $state(true);
  let autoStart = $state(false);
  let systemPrompt = $state("Keep your responses as concise, precise, to the point.\nAnswer the question in as few words as possible.\nNo Yapping.");
  let isLoading = $state(true);
  let saveMessage = $state("");
  let showSaveMessage = $state(false);

  const models = [
    { value: "openai/gpt-oss-120b", label: "GPT OSS 120B" },
    { value: "openai/gpt-4o", label: "GPT-4o" },
    { value: "anthropic/claude-3.5-sonnet", label: "Claude 3.5 Sonnet" },
    { value: "google/gemini-pro", label: "Gemini Pro" },
  ];

  function getModelLabel(value: string): string {
    return models.find((m) => m.value === value)?.label ?? value;
  }

  onMount(async () => {
    try {
      const settings = await invoke<Settings>("load_settings");
      apiKey = settings.apiKey;
      selectedModel = settings.selectedModel;
      darkMode = settings.darkMode;
      autoStart = settings.autoStart;
      systemPrompt = settings.systemPrompt || "";
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
          systemPrompt,
        },
      });
      console.log("Settings saved:", { apiKey, selectedModel, darkMode, autoStart, systemPrompt });
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
      </section>
    </main>

    <footer class="settings-footer">
      <Button variant="outline" onclick={handleCancel}>Cancel</Button>
      <Button onclick={saveSettings}>Save</Button>
    </footer>

    {#if showSaveMessage}
      <div class="save-toast" class:error={saveMessage.includes('Failed')}>
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
