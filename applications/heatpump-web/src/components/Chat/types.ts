export type ChatRole = 'user' | 'assistant';

export interface ToolCall {
  id: string;
  name: string;
  input: unknown;
  result?: ToolResult;
}

export interface ToolResult {
  id: string;
  output: unknown;
  is_error?: boolean;
}

export interface ChatMessage {
  id: string;
  role: ChatRole;
  content: string;
  tool_calls: ToolCall[];
  is_streaming?: boolean;
}

export interface TokenEvent {
  type: 'token';
  text: string;
}

export interface ToolCallEvent {
  type: 'tool_call';
  id: string;
  name: string;
  input: unknown;
}

export interface ToolResultEvent {
  type: 'tool_result';
  id: string;
  output: unknown;
  is_error?: boolean;
}

export interface DoneEvent {
  type: 'done';
}

export interface ChatErrorEvent {
  type: 'error';
  message: string;
}

export type ChatEvent = TokenEvent | ToolCallEvent | ToolResultEvent | DoneEvent | ChatErrorEvent;

export interface ChatRequestMessage {
  role: ChatRole;
  content: string;
}
