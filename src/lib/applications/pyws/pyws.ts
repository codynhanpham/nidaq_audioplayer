/** 
A WRAPPER FOR WEBSOCKET COMMUNICATION WITH PYTHON

This module works along side the main Tauri app and relies on
the Tauri Websocket Plugin (https://tauri.app/plugin/websocket/)
to facilitate communication between the frontend and the Python backend.
*/

import WebSocket from "@tauri-apps/plugin-websocket";

export interface PYWSOptions {
  reconnectAttempts?: number;
  reconnectDelay?: number;
  healthcheckTimeout?: number;
}

export interface PYWSMessage {
  type: string;
  data: any;
  timestamp?: number;
}

export class PYWS {
  private ws: WebSocket | null = null;
  private url: string;
  private isConnected: boolean = false;
  private messageListeners: ((msg: any) => void)[] = [];
  private options: Required<PYWSOptions>;
  private reconnectTimer: number | null = null;
  private currentReconnectAttempts: number = 0;

  constructor(url: string, options: PYWSOptions = {}) {
    this.url = url;
    this.options = {
      reconnectAttempts: options.reconnectAttempts ?? 3,
      reconnectDelay: options.reconnectDelay ?? 1000,
      healthcheckTimeout: options.healthcheckTimeout ?? 5000,
    };
  }

  /**
   * Connect to the WebSocket server
   */
  async connect(): Promise<boolean> {
    try {
      this.ws = await WebSocket.connect(this.url);
      this.isConnected = true;
      this.currentReconnectAttempts = 0;

      // Set up message listener
      this.ws.addListener((msg) => {
        this.messageListeners.forEach((listener) => listener(msg));
      });

      return true;
    } catch (error) {
      console.error("Failed to connect to WebSocket:", error);
      this.isConnected = false;
      return false;
    }
  }

  /**
   * Disconnect from the WebSocket server
   */
  async disconnect(): Promise<void> {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.ws) {
      await this.ws.disconnect();
      this.ws = null;
    }
    this.isConnected = false;
  }

  /**
   * Check if the WebSocket connection is healthy
   * Sends a healthcheck message and waits for response
   */
  async connected(): Promise<boolean> {
    if (!this.ws || !this.isConnected) {
      return false;
    }

    return new Promise((resolve) => {
      const timeout = setTimeout(() => {
        resolve(false);
      }, this.options.healthcheckTimeout);

      // Listen for healthcheck response
      const healthcheckListener = (msg: any) => {
        if (msg.type === "healthcheck_response" || msg === "healthcheck_ok") {
          clearTimeout(timeout);
          this.removeListener(healthcheckListener);
          resolve(true);
        }
      };

      this.addListener(healthcheckListener);

      // Send healthcheck message
      this.send("!#healthcheck").catch(() => {
        clearTimeout(timeout);
        this.removeListener(healthcheckListener);
        resolve(false);
      });
    });
  }

  /**
   * Send a message to the WebSocket server
   */
  async send(message: string | PYWSMessage): Promise<boolean> {
    if (!this.ws || !this.isConnected) {
      console.warn("WebSocket not connected. Attempting to reconnect...");
      const reconnected = await this.attemptReconnect();
      if (!reconnected) {
        return false;
      }
    }

    try {
      const msgToSend =
        typeof message === "string" ? message : JSON.stringify(message);
      await this.ws!.send(msgToSend);
      return true;
    } catch (error) {
      console.error("Failed to send message:", error);
      this.isConnected = false;
      return false;
    }
  }

  /**
   * Add a message listener
   */
  addListener(listener: (msg: any) => void): void {
    this.messageListeners.push(listener);
  }

  /**
   * Remove a message listener
   */
  removeListener(listener: (msg: any) => void): void {
    const index = this.messageListeners.indexOf(listener);
    if (index > -1) {
      this.messageListeners.splice(index, 1);
    }
  }

  /**
   * Remove all message listeners
   */
  removeAllListeners(): void {
    this.messageListeners = [];
  }

  /**
   * Get connection status
   */
  get connectionStatus(): boolean {
    return this.isConnected;
  }

  /**
   * Get the WebSocket URL
   */
  get socketUrl(): string {
    return this.url;
  }

  /**
   * Attempt to reconnect to the WebSocket
   */
  private async attemptReconnect(): Promise<boolean> {
    if (this.currentReconnectAttempts >= this.options.reconnectAttempts) {
      console.error("Max reconnection attempts reached");
      return false;
    }

    this.currentReconnectAttempts++;
    console.log(
      `Attempting to reconnect (${this.currentReconnectAttempts}/${this.options.reconnectAttempts})...`
    );

    await new Promise((resolve) =>
      setTimeout(resolve, this.options.reconnectDelay)
    );

    return await this.connect();
  }

  /**
   * Send a structured message with type and data
   */
  async sendMessage(type: string, data: any): Promise<boolean> {
    const message: PYWSMessage = {
      type,
      data,
      timestamp: Date.now(),
    };
    return await this.send(message);
  }

  /**
   * Send a command and wait for a specific response
   */
  async sendCommand(
    command: string,
    expectedResponseType?: string,
    timeout: number = 5000
  ): Promise<any> {
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.removeListener(responseListener);
        reject(new Error(`Command timeout: ${command}`));
      }, timeout);

      const responseListener = (msg: any) => {
        let parsedMsg;
        try {
          parsedMsg = typeof msg === "string" ? JSON.parse(msg) : msg;
        } catch {
          parsedMsg = { type: "raw", data: msg };
        }

        if (!expectedResponseType || parsedMsg.type === expectedResponseType) {
          clearTimeout(timer);
          this.removeListener(responseListener);
          resolve(parsedMsg);
        }
      };

      this.addListener(responseListener);

      this.send(command).catch((error) => {
        clearTimeout(timer);
        this.removeListener(responseListener);
        reject(error);
      });
    });
  }
}

/*
EXAMPLE USAGE:

// Create a new PYWS instance
const pyws = new PYWS("ws://127.0.0.1:8080");

// Connect to the WebSocket
await pyws.connect();

// Check if the connection is healthy (sends "!#healthcheck" and waits for response)
const isConnected = await pyws.connected();
console.log("Connection healthy:", isConnected);

// Send a simple message
await pyws.send("Hello World!");

// Send a structured message
await pyws.sendMessage("audio_command", { action: "play", file: "audio.wav" });

// Listen for messages
pyws.addListener((msg) => {
  console.log('Received Message:', msg);
});

// Send a command and wait for specific response
try {
  const response = await pyws.sendCommand("get_status", "status_response", 3000);
  console.log("Status:", response.data);
} catch (error) {
  console.error("Command failed:", error);
}

// Disconnect when done
await pyws.disconnect();
*/
