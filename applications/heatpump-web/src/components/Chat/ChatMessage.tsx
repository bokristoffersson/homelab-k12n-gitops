import { useState } from 'react';
import { ChatMessage as ChatMessageType, ToolCall } from './types';

function formatJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function ToolCallCard({ call }: { call: ToolCall }) {
  const [expanded, setExpanded] = useState(false);
  const hasResult = call.result !== undefined;
  const isError = call.result?.is_error === true;

  return (
    <div className={`tool-card ${isError ? 'tool-card-error' : ''}`}>
      <button
        type="button"
        className="tool-card-header"
        onClick={() => setExpanded((prev) => !prev)}
      >
        <span className={`tool-card-toggle ${expanded ? 'expanded' : ''}`}>&#9654;</span>
        <span className="tool-card-name">{call.name}</span>
        <span className="tool-card-status">
          {hasResult ? (isError ? 'error' : 'done') : 'running...'}
        </span>
      </button>
      {expanded && (
        <div className="tool-card-body">
          <div className="tool-card-section">
            <div className="tool-card-label">Input</div>
            <pre className="tool-card-pre">{formatJson(call.input)}</pre>
          </div>
          {hasResult && call.result && (
            <div className="tool-card-section">
              <div className="tool-card-label">
                {isError ? 'Error' : 'Output'}
              </div>
              <pre className="tool-card-pre">{formatJson(call.result.output)}</pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default function ChatMessage({ message }: { message: ChatMessageType }) {
  const isUser = message.role === 'user';
  const hasContent = message.content.length > 0;

  return (
    <div className={`chat-message ${isUser ? 'chat-message-user' : 'chat-message-assistant'}`}>
      <div className="chat-message-bubble">
        {message.tool_calls.length > 0 && (
          <div className="chat-message-tools">
            {message.tool_calls.map((call) => (
              <ToolCallCard key={call.id} call={call} />
            ))}
          </div>
        )}
        {hasContent && <div className="chat-message-text">{message.content}</div>}
        {!hasContent && message.is_streaming && message.tool_calls.length === 0 && (
          <div className="chat-message-thinking">thinking...</div>
        )}
      </div>
    </div>
  );
}
