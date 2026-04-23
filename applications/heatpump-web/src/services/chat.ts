import {
  ChatErrorEvent,
  ChatEvent,
  ChatRequestMessage,
  DoneEvent,
  TokenEvent,
  ToolCallEvent,
  ToolResultEvent,
} from '../components/Chat/types';

const API_BASE_URL = window.ENV?.API_URL || 'https://homelab.k12n.com';

export interface StreamChatHandlers {
  onToken: (event: TokenEvent) => void;
  onToolCall: (event: ToolCallEvent) => void;
  onToolResult: (event: ToolResultEvent) => void;
  onDone: (event: DoneEvent) => void;
  onError: (event: ChatErrorEvent) => void;
}

interface SSEFrame {
  event: string;
  data: string;
}

function parseFrame(raw: string): SSEFrame | null {
  const lines = raw.split('\n');
  let event = 'message';
  const dataLines: string[] = [];

  for (const line of lines) {
    if (line.startsWith(':')) {
      continue;
    }
    if (line.startsWith('event:')) {
      event = line.slice('event:'.length).trim();
    } else if (line.startsWith('data:')) {
      dataLines.push(line.slice('data:'.length).trim());
    }
  }

  if (dataLines.length === 0) {
    return null;
  }

  return { event, data: dataLines.join('\n') };
}

function dispatch(frame: SSEFrame, handlers: StreamChatHandlers): void {
  let payload: unknown;
  try {
    payload = JSON.parse(frame.data);
  } catch {
    handlers.onError({ type: 'error', message: `Invalid SSE payload: ${frame.data}` });
    return;
  }

  const typed = payload as Partial<ChatEvent> & Record<string, unknown>;

  switch (frame.event) {
    case 'token':
      if (typeof typed.text === 'string') {
        handlers.onToken({ type: 'token', text: typed.text });
      }
      return;
    case 'tool_call':
      if (typeof typed.id === 'string' && typeof typed.name === 'string') {
        handlers.onToolCall({
          type: 'tool_call',
          id: typed.id,
          name: typed.name,
          input: typed.input,
        });
      }
      return;
    case 'tool_result':
      if (typeof typed.id === 'string') {
        handlers.onToolResult({
          type: 'tool_result',
          id: typed.id,
          output: typed.output,
          is_error: typeof typed.is_error === 'boolean' ? typed.is_error : false,
        });
      }
      return;
    case 'done':
      handlers.onDone({ type: 'done' });
      return;
    case 'error':
      handlers.onError({
        type: 'error',
        message: typeof typed.message === 'string' ? typed.message : 'Unknown stream error',
      });
      return;
    default:
      return;
  }
}

export async function streamChat(
  messages: ChatRequestMessage[],
  handlers: StreamChatHandlers,
  signal?: AbortSignal,
): Promise<void> {
  let response: Response;
  try {
    response = await fetch(`${API_BASE_URL}/api/v1/chat`, {
      method: 'POST',
      credentials: 'include',
      headers: {
        'Content-Type': 'application/json',
        Accept: 'text/event-stream',
      },
      body: JSON.stringify({ messages }),
      ...(signal ? { signal } : {}),
    });
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Network error';
    handlers.onError({ type: 'error', message });
    return;
  }

  if (!response.ok) {
    handlers.onError({
      type: 'error',
      message: `Chat request failed: ${response.status} ${response.statusText}`,
    });
    return;
  }

  if (!response.body) {
    handlers.onError({ type: 'error', message: 'Empty response body' });
    return;
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder('utf-8');
  let buffer = '';
  let sawDone = false;

  try {
    for (;;) {
      const { value, done } = await reader.read();
      if (done) {
        break;
      }
      buffer += decoder.decode(value, { stream: true });

      let separatorIndex = buffer.indexOf('\n\n');
      while (separatorIndex !== -1) {
        const rawFrame = buffer.slice(0, separatorIndex);
        buffer = buffer.slice(separatorIndex + 2);
        const frame = parseFrame(rawFrame);
        if (frame) {
          if (frame.event === 'done') {
            sawDone = true;
          }
          dispatch(frame, handlers);
        }
        separatorIndex = buffer.indexOf('\n\n');
      }
    }

    const trailing = buffer.trim();
    if (trailing.length > 0) {
      const frame = parseFrame(trailing);
      if (frame) {
        if (frame.event === 'done') {
          sawDone = true;
        }
        dispatch(frame, handlers);
      }
    }

    if (!sawDone) {
      handlers.onError({ type: 'error', message: 'Stream ended without done event' });
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Stream read error';
    handlers.onError({ type: 'error', message });
  } finally {
    reader.releaseLock();
  }
}
