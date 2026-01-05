// ============================================================
// WebRTC Signaling Types
// ============================================================

export interface SdpOffer {
  type: 'offer';
  sdp: string;
}

export interface SdpAnswer {
  type: 'answer';
  sdp: string;
  session_id: string;
}

export interface IceCandidate {
  candidate: string;
  sdpMLineIndex?: number;
  sdpMid?: string;
  usernameFragment?: string;
}

export interface WebRtcStatus {
  state: string;
  ice_gathering_state?: string;
  ice_connection_state?: string;
  local_candidate_count: number;
}

// ============================================================
// WebSocket Message Types (for status/transcripts/responses)
// ============================================================

// Server -> Client messages
export interface WsSessionInfoMessage {
  type: 'session_info';
  session_id: string;
}

export interface WsStatusMessage {
  type: 'status';
  state: 'active' | 'listening' | 'processing' | 'thinking' | 'idle';
  stage: string;
}

export interface WsTranscriptMessage {
  type: 'transcript';
  text: string;
  is_final: boolean;
}

export interface WsResponseMessage {
  type: 'response';
  text: string;
}

export interface WsResponseAudioMessage {
  type: 'response_audio';
  data: string; // base64 PCM audio (fallback if WebRTC fails)
}

export interface WsErrorMessage {
  type: 'error';
  message: string;
}

export interface WsPongMessage {
  type: 'pong';
}

export type ServerWsMessage =
  | WsSessionInfoMessage
  | WsStatusMessage
  | WsTranscriptMessage
  | WsResponseMessage
  | WsResponseAudioMessage
  | WsErrorMessage
  | WsPongMessage;

// Client -> Server messages
export interface WsTextMessage {
  type: 'text';
  content: string;
}

export interface WsPingMessage {
  type: 'ping';
}

export interface WsEndSessionMessage {
  type: 'end_session';
}

export type ClientWsMessage = WsTextMessage | WsPingMessage | WsEndSessionMessage;

// ============================================================
// Customer Data Types
// ============================================================

export type CustomerSegment = 'high_value' | 'trust_seeker' | 'shakti' | 'young_pro';
export type Provider = 'muthoot' | 'manappuram' | 'iifl' | 'other';

export interface Customer {
  id: string;
  name: string;
  language: string;
  segment: CustomerSegment;
  current_provider: Provider;
  estimated_outstanding: number;
  estimated_rate: number;
  city: string;
  phone?: string;
}

export interface Language {
  code: string;
  name: string;
  native: string;
}

// ============================================================
// Conversation Types
// ============================================================

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  isPartial?: boolean;
}

// Conversation stages from the agent FSM
export type ConversationStage =
  | 'Greeting'
  | 'Discovery'
  | 'Qualification'
  | 'Presentation'
  | 'ObjectionHandling'
  | 'Closing'
  | 'Farewell';

// ============================================================
// Connection State Types
// ============================================================

export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error';
export type AgentState = 'idle' | 'listening' | 'processing' | 'speaking';
export type VadState = 'silence' | 'speech_detected' | 'speech_active' | 'speech_ended';

export interface SessionState {
  sessionId: string | null;
  status: ConnectionStatus;
  agentState: AgentState;
  vadState: VadState;
  stage: string;
  error: string | null;
}

// ============================================================
// Metrics Types
// ============================================================

export interface Metrics {
  // Client-side metrics
  audioInputLevel: number; // 0-1 normalized
  audioOutputLevel: number;

  // WebRTC stats
  rtcConnectionState: string;
  iceConnectionState: string;

  // Latency metrics (from server)
  asrLatencyMs?: number;
  llmLatencyMs?: number;
  ttsLatencyMs?: number;
  totalLatencyMs?: number;

  // Session stats
  turnCount: number;
  sessionDurationMs: number;
}

// ============================================================
// API Response Types
// ============================================================

export interface CreateSessionResponse {
  session_id: string;
  websocket_url: string;
  rag_enabled: boolean;
  tools_wired: boolean;
}

export interface SessionInfoResponse {
  session_id: string;
  active: boolean;
  stage: string;
  turn_count: number;
}

export interface HealthCheckResponse {
  status: 'healthy' | 'degraded';
  version: string;
  checks: Record<string, { status: string; [key: string]: unknown }>;
}

// ============================================================
// Hook Return Types
// ============================================================

export interface UseVoiceAgentReturn {
  // Connection state
  sessionId: string | null;
  connectionStatus: ConnectionStatus;
  agentState: AgentState;
  vadState: VadState;
  stage: string;
  error: string | null;

  // Conversation
  messages: Message[];
  currentTranscript: string; // Partial transcript being received

  // Metrics
  metrics: Metrics;

  // Audio state
  isMuted: boolean;
  inputLevel: number;
  isSpeaking: boolean; // Agent is speaking

  // Actions
  connect: (customerId: string, language: string) => Promise<void>;
  disconnect: () => void;
  sendText: (text: string) => void;
  toggleMute: () => void;
}
