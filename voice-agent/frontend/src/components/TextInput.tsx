import React, { useState, useCallback } from 'react';
import { theme } from '../styles/theme';

interface TextInputProps {
  onSend: (text: string) => void;
  disabled?: boolean;
  placeholder?: string;
}

export const TextInput: React.FC<TextInputProps> = ({
  onSend,
  disabled = false,
  placeholder = 'Type a message...',
}) => {
  const [text, setText] = useState('');

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      const trimmed = text.trim();
      if (trimmed && !disabled) {
        onSend(trimmed);
        setText('');
      }
    },
    [text, disabled, onSend]
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSubmit(e);
      }
    },
    [handleSubmit]
  );

  return (
    <form style={styles.container} onSubmit={handleSubmit}>
      <input
        type="text"
        value={text}
        onChange={e => setText(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        disabled={disabled}
        style={{
          ...styles.input,
          ...(disabled ? styles.inputDisabled : {}),
        }}
      />
      <button
        type="submit"
        disabled={disabled || !text.trim()}
        style={{
          ...styles.button,
          ...(disabled || !text.trim() ? styles.buttonDisabled : {}),
        }}
      >
        Send
      </button>
    </form>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    gap: theme.spacing.sm,
    padding: theme.spacing.md,
    background: theme.colors.bgSecondary,
    borderTop: `1px solid ${theme.colors.border}`,
  },
  input: {
    flex: 1,
    padding: `${theme.spacing.sm} ${theme.spacing.md}`,
    fontSize: theme.fontSize.md,
    color: theme.colors.textPrimary,
    background: theme.colors.bgTertiary,
    border: `1px solid ${theme.colors.border}`,
    borderRadius: theme.borderRadius.md,
    outline: 'none',
    transition: `border-color ${theme.transitions.fast}`,
  },
  inputDisabled: {
    opacity: 0.5,
    cursor: 'not-allowed',
  },
  button: {
    padding: `${theme.spacing.sm} ${theme.spacing.lg}`,
    fontSize: theme.fontSize.md,
    fontWeight: theme.fontWeight.semibold,
    color: theme.colors.textPrimary,
    background: theme.colors.kotakRed,
    border: 'none',
    borderRadius: theme.borderRadius.md,
    cursor: 'pointer',
    transition: `background ${theme.transitions.fast}`,
  },
  buttonDisabled: {
    opacity: 0.5,
    cursor: 'not-allowed',
  },
};
