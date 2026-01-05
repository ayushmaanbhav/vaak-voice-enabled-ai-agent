import React, { useEffect, useRef } from 'react';
import ReactMarkdown from 'react-markdown';
import { theme } from '../styles/theme';
import type { Message } from '../types';

interface ConversationProps {
  messages: Message[];
  currentTranscript: string;
}

export const Conversation: React.FC<ConversationProps> = ({ messages, currentTranscript }) => {
  const containerRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom on new messages
  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [messages, currentTranscript]);

  return (
    <div style={styles.container} ref={containerRef}>
      {messages.length === 0 && !currentTranscript && (
        <div style={styles.empty}>
          <div style={styles.emptyIcon}>🎙️</div>
          <div style={styles.emptyText}>Conversation will appear here</div>
          <div style={styles.emptySubtext}>Start speaking to begin</div>
        </div>
      )}

      {messages.map(message => (
        <MessageBubble key={message.id} message={message} />
      ))}

      {currentTranscript && (
        <div style={styles.transcriptContainer}>
          <div style={styles.transcriptLabel}>Listening...</div>
          <div style={styles.transcript}>{currentTranscript}</div>
        </div>
      )}
    </div>
  );
};

interface MessageBubbleProps {
  message: Message;
}

const MessageBubble: React.FC<MessageBubbleProps> = ({ message }) => {
  const isUser = message.role === 'user';
  const isSystem = message.role === 'system';

  if (isSystem) {
    return (
      <div style={styles.systemMessage}>
        <span style={styles.systemText}>{message.content}</span>
      </div>
    );
  }

  return (
    <div
      style={{
        ...styles.messageRow,
        justifyContent: isUser ? 'flex-end' : 'flex-start',
      }}
    >
      {!isUser && <div style={styles.avatar}>🤖</div>}

      <div
        style={{
          ...styles.bubble,
          ...(isUser ? styles.userBubble : styles.assistantBubble),
          ...(message.isPartial ? styles.partialBubble : {}),
        }}
      >
        <div style={styles.messageContent}>
          {isUser ? (
            message.content
          ) : (
            <ReactMarkdown
              components={{
                // Style headings
                h1: ({ children }) => <h1 style={styles.mdH1}>{children}</h1>,
                h2: ({ children }) => <h2 style={styles.mdH2}>{children}</h2>,
                h3: ({ children }) => <h3 style={styles.mdH3}>{children}</h3>,
                // Style paragraphs
                p: ({ children }) => <p style={styles.mdP}>{children}</p>,
                // Style lists
                ul: ({ children }) => <ul style={styles.mdUl}>{children}</ul>,
                ol: ({ children }) => <ol style={styles.mdOl}>{children}</ol>,
                li: ({ children }) => <li style={styles.mdLi}>{children}</li>,
                // Style bold/italic
                strong: ({ children }) => <strong style={styles.mdStrong}>{children}</strong>,
                em: ({ children }) => <em style={styles.mdEm}>{children}</em>,
                // Style code
                code: ({ children }) => <code style={styles.mdCode}>{children}</code>,
              }}
            >
              {message.content}
            </ReactMarkdown>
          )}
        </div>
        <div style={styles.timestamp}>
          {new Date(message.timestamp).toLocaleTimeString([], {
            hour: '2-digit',
            minute: '2-digit',
          })}
          {message.isPartial && <span style={styles.typing}> ...</span>}
        </div>
      </div>

      {isUser && <div style={styles.avatar}>👤</div>}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    flex: 1,
    overflowY: 'auto',
    overflowX: 'hidden',
    padding: theme.spacing.lg,
    display: 'flex',
    flexDirection: 'column',
    gap: theme.spacing.md,
    minHeight: 0, // Critical for flex overflow to work
  },
  empty: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    color: theme.colors.textMuted,
  },
  emptyIcon: {
    fontSize: '48px',
    marginBottom: theme.spacing.md,
    opacity: 0.5,
  },
  emptyText: {
    fontSize: theme.fontSize.lg,
    fontWeight: theme.fontWeight.medium,
  },
  emptySubtext: {
    fontSize: theme.fontSize.sm,
    marginTop: theme.spacing.xs,
    opacity: 0.7,
  },
  messageRow: {
    display: 'flex',
    alignItems: 'flex-end',
    gap: theme.spacing.sm,
  },
  avatar: {
    width: '36px',
    height: '36px',
    borderRadius: '50%',
    background: theme.colors.bgTertiary,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '18px',
    flexShrink: 0,
  },
  bubble: {
    maxWidth: '70%',
    padding: `${theme.spacing.sm} ${theme.spacing.md}`,
    borderRadius: theme.borderRadius.lg,
    position: 'relative',
  },
  userBubble: {
    background: theme.colors.kotakRed,
    color: theme.colors.textPrimary,
    borderBottomRightRadius: theme.borderRadius.sm,
  },
  assistantBubble: {
    background: theme.colors.bgTertiary,
    color: theme.colors.textPrimary,
    borderBottomLeftRadius: theme.borderRadius.sm,
  },
  partialBubble: {
    opacity: 0.9,
    borderStyle: 'dashed',
    borderWidth: '1px',
    borderColor: theme.colors.border,
  },
  messageContent: {
    fontSize: theme.fontSize.md,
    lineHeight: 1.7, // Increased for Devanagari matras
    wordBreak: 'break-word',
    unicodeBidi: 'plaintext', // Proper Unicode handling
    whiteSpace: 'pre-wrap', // Preserve spaces
  },
  timestamp: {
    fontSize: theme.fontSize.xs,
    opacity: 0.6,
    marginTop: theme.spacing.xs,
    textAlign: 'right',
  },
  typing: {
    animation: 'blink 1s infinite',
  },
  transcriptContainer: {
    padding: theme.spacing.md,
    background: 'rgba(237, 28, 36, 0.1)',
    borderRadius: theme.borderRadius.md,
    border: `1px dashed ${theme.colors.kotakRed}`,
  },
  transcriptLabel: {
    fontSize: theme.fontSize.xs,
    color: theme.colors.kotakRed,
    fontWeight: theme.fontWeight.semibold,
    marginBottom: theme.spacing.xs,
  },
  transcript: {
    fontSize: theme.fontSize.lg, // Larger for better Hindi visibility
    color: theme.colors.textPrimary,
    fontStyle: 'normal', // Remove italic for better Devanagari
    lineHeight: 1.8, // Extra height for complex scripts
    unicodeBidi: 'plaintext',
  },
  systemMessage: {
    textAlign: 'center',
    padding: theme.spacing.sm,
  },
  systemText: {
    fontSize: theme.fontSize.sm,
    color: theme.colors.textMuted,
    background: theme.colors.bgCard,
    padding: `${theme.spacing.xs} ${theme.spacing.md}`,
    borderRadius: theme.borderRadius.full,
  },
  // Markdown styles for formatted responses
  mdH1: {
    fontSize: theme.fontSize.xl,
    fontWeight: theme.fontWeight.bold,
    margin: `${theme.spacing.md} 0 ${theme.spacing.sm} 0`,
    lineHeight: 1.3,
  },
  mdH2: {
    fontSize: theme.fontSize.lg,
    fontWeight: theme.fontWeight.semibold,
    margin: `${theme.spacing.md} 0 ${theme.spacing.sm} 0`,
    lineHeight: 1.3,
  },
  mdH3: {
    fontSize: theme.fontSize.md,
    fontWeight: theme.fontWeight.semibold,
    margin: `${theme.spacing.sm} 0 ${theme.spacing.xs} 0`,
    lineHeight: 1.3,
  },
  mdP: {
    margin: `${theme.spacing.xs} 0`,
    lineHeight: 1.7,
  },
  mdUl: {
    margin: `${theme.spacing.xs} 0`,
    paddingLeft: theme.spacing.lg,
    listStyleType: 'disc',
  },
  mdOl: {
    margin: `${theme.spacing.xs} 0`,
    paddingLeft: theme.spacing.lg,
    listStyleType: 'decimal',
  },
  mdLi: {
    margin: `${theme.spacing.xs} 0`,
    lineHeight: 1.6,
  },
  mdStrong: {
    fontWeight: theme.fontWeight.bold,
  },
  mdEm: {
    fontStyle: 'italic',
  },
  mdCode: {
    background: 'rgba(0, 0, 0, 0.2)',
    padding: '2px 6px',
    borderRadius: theme.borderRadius.sm,
    fontFamily: 'monospace',
    fontSize: '0.9em',
  },
};
