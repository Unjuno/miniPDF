/**
 * ログシステム
 * why: 構造化ログでバグを特定しやすくする
 * alt: console.logを直接使用（ログレベル制御ができない）
 * evidence: 構造化ログにより、ログレベルでフィルタリングできる
 */

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
}

interface LogEntry {
  level: LogLevel;
  message: string;
  timestamp: string;
  context?: Record<string, unknown>;
  error?: Error;
}

class Logger {
  private level: LogLevel;
  private logs: LogEntry[] = [];
  private maxLogs = 1000; // メモリリークを防ぐため、最大ログ数を制限

  constructor() {
    // why: 本番環境ではWARN以上、開発環境ではDEBUG以上を表示
    // alt: 常にすべてのログを表示（パフォーマンスの問題）
    // evidence: ログレベルを制御することで、パフォーマンスとデバッグ性のバランスを取る
    this.level = import.meta.env.DEV ? LogLevel.DEBUG : LogLevel.WARN;
  }

  private log(level: LogLevel, message: string, context?: Record<string, unknown>, error?: Error): void {
    if (level < this.level) {
      return;
    }

    const entry: LogEntry = {
      level,
      message,
      timestamp: new Date().toISOString(),
      context,
      error,
    };

    this.logs.push(entry);
    if (this.logs.length > this.maxLogs) {
      this.logs.shift(); // 古いログを削除
    }

    const logMessage = `[${entry.timestamp}] [${LogLevel[level]}] ${message}`;
    const logData = context || error ? { context, error: error?.stack } : undefined;

    switch (level) {
      case LogLevel.DEBUG:
        console.debug(logMessage, logData || '');
        break;
      case LogLevel.INFO:
        console.info(logMessage, logData || '');
        break;
      case LogLevel.WARN:
        console.warn(logMessage, logData || '');
        break;
      case LogLevel.ERROR:
        console.error(logMessage, logData || '');
        if (error) {
          console.error('Error stack:', error.stack);
        }
        break;
    }
  }

  debug(message: string, context?: Record<string, unknown>): void {
    this.log(LogLevel.DEBUG, message, context);
  }

  info(message: string, context?: Record<string, unknown>): void {
    this.log(LogLevel.INFO, message, context);
  }

  warn(message: string, context?: Record<string, unknown>): void {
    this.log(LogLevel.WARN, message, context);
  }

  error(message: string, error?: Error, context?: Record<string, unknown>): void {
    this.log(LogLevel.ERROR, message, context, error);
  }

  // why: ログを取得してエラーレポートに含める
  // alt: ログを取得できない（エラーレポートに情報が不足）
  // evidence: ログを取得することで、エラー発生時の状況を把握できる
  getLogs(level?: LogLevel): LogEntry[] {
    if (level !== undefined) {
      return this.logs.filter(log => log.level >= level);
    }
    return [...this.logs];
  }

  // why: ログをクリアしてメモリを解放
  // alt: ログを保持し続ける（メモリリークの可能性）
  // evidence: ログをクリアすることで、メモリ使用量を制御できる
  clear(): void {
    this.logs = [];
  }

  // why: ログレベルを動的に変更
  // alt: ログレベルを固定（デバッグ時に不便）
  // evidence: ログレベルを動的に変更することで、デバッグ時に詳細なログを取得できる
  setLevel(level: LogLevel): void {
    this.level = level;
  }
}

export const logger = new Logger();

