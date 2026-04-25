import { KeyboardEvent, useCallback, useEffect, useRef, useState } from 'react';
import { streamChat } from '../../services/chat';
import ChatMessage from './ChatMessage';
import { ChatMessage as ChatMessageType, ChatRequestMessage, ToolCall } from './types';
import './Chat.css';

function createId(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID();
  }
  return `${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

function toRequestMessages(messages: ChatMessageType[]): ChatRequestMessage[] {
  return messages
    .filter((m) => m.content.length > 0)
    .map((m) => ({ role: m.role, content: m.content }));
}

export default function Chat() {
  const [messages, setMessages] = useState<ChatMessageType[]>([]);
  const [input, setInput] = useState('');
  const [isStreaming, setIsStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const scrollRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const node = scrollRef.current;
    if (node) {
      node.scrollTop = node.scrollHeight;
    }
  }, [messages]);

  const handleSubmit = useCallback(() => {
    const trimmed = input.trim();
    if (!trimmed || isStreaming) {
      return;
    }

    const userMessage: ChatMessageType = {
      id: createId(),
      role: 'user',
      content: trimmed,
      tool_calls: [],
    };

    const assistantId = createId();
    const assistantMessage: ChatMessageType = {
      id: assistantId,
      role: 'assistant',
      content: '',
      tool_calls: [],
      is_streaming: true,
    };

    const nextMessages = [...messages, userMessage, assistantMessage];
    setMessages(nextMessages);
    setInput('');
    setError(null);
    setIsStreaming(true);

    const requestMessages = toRequestMessages([...messages, userMessage]);

    const updateAssistant = (
      updater: (current: ChatMessageType) => ChatMessageType,
    ): void => {
      setMessages((prev) =>
        prev.map((m) => (m.id === assistantId ? updater(m) : m)),
      );
    };

    void streamChat(requestMessages, {
      onToken: (event) => {
        updateAssistant((current) => ({
          ...current,
          content: current.content + event.text,
        }));
      },
      onToolCall: (event) => {
        const call: ToolCall = {
          id: event.id,
          name: event.name,
          input: event.input,
        };
        updateAssistant((current) => ({
          ...current,
          tool_calls: [...current.tool_calls, call],
        }));
      },
      onToolResult: (event) => {
        updateAssistant((current) => ({
          ...current,
          tool_calls: current.tool_calls.map((call) =>
            call.id === event.id
              ? {
                  ...call,
                  result: {
                    id: event.id,
                    output: event.output,
                    is_error: event.is_error ?? false,
                  },
                }
              : call,
          ),
        }));
      },
      onDone: () => {
        updateAssistant((current) => ({ ...current, is_streaming: false }));
        setIsStreaming(false);
      },
      onError: (event) => {
        setError(event.message);
        updateAssistant((current) => ({ ...current, is_streaming: false }));
        setIsStreaming(false);
      },
    });
  }, [input, isStreaming, messages]);

  const handleKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>): void => {
    if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
      event.preventDefault();
      handleSubmit();
    }
  };

  const handleClear = (): void => {
    if (isStreaming) {
      return;
    }
    setMessages([]);
    setError(null);
  };

  const hasMessages = messages.length > 0;

  return (
    <div className="chat-page">
      <div className="chat-header">
        <h2>Chat</h2>
        <button
          type="button"
          className="chat-clear-button"
          onClick={handleClear}
          disabled={isStreaming || !hasMessages}
        >
          Clear
        </button>
      </div>

      {error && (
        <div className="chat-error-banner">
          <strong>Chat error:</strong>
          <div>{error}</div>
        </div>
      )}

      <div className="chat-messages" ref={scrollRef}>
        {!hasMessages && (
          <div className="chat-empty">
            Ask a question about your homelab. Press Cmd/Ctrl+Enter to submit.
          </div>
        )}
        {messages.map((message) => (
          <ChatMessage key={message.id} message={message} />
        ))}
        {isStreaming && <div className="chat-thinking">thinking...</div>}
      </div>

      <form
        className="chat-input-form"
        onSubmit={(event) => {
          event.preventDefault();
          handleSubmit();
        }}
      >
        <textarea
          className="chat-input"
          value={input}
          onChange={(event) => setInput(event.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Ask something..."
          rows={3}
          disabled={isStreaming}
        />
        <button
          type="submit"
          className="chat-submit-button"
          disabled={isStreaming || input.trim().length === 0}
        >
          {isStreaming ? 'Sending...' : 'Send'}
        </button>
      </form>
    </div>
  );
}
