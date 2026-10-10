import { API_ROOT } from "@/api";
import type { WSClientMessage, WSServerMessage, WSServerMessageType } from "@/api/types";
import { onScopeDispose, ref, type Ref } from "vue";
import * as Y from 'yjs';
import * as awarenessProtocol from 'y-protocols/awareness.js'
import { IndexeddbPersistence } from 'y-indexeddb'

const openSockets: Map<string, EditorWS> = new Map();

export interface EditorWS {
  state: Ref<ConnectionState>,
  doc: Y.Doc,
  awareness: awarenessProtocol.Awareness,
  connect: () => void,
  disconnect: () => void,
  send: (msg: WSClientMessage) => void,
  on: <T extends WSServerMessageType>(eventType: T, f: (message: Extract<WSServerMessage, { t: T }>) => void) => () => void,
}

enum ConnectionState {
  Disconnected,
  /// Setting up connection or reconnecting
  EstablishingConnection,
  Connected,
}

export const useEditorWS = (projectId: string): EditorWS => {
  const existing = openSockets.get(projectId);
  if (existing) return existing;

  let websocket: WebSocket | null = null;
  const state = ref(ConnectionState.Disconnected);
  const shouldBeConnected = ref(false);

  const doc = new Y.Doc();
  const awareness = new awarenessProtocol.Awareness(doc);
  const persistence = new IndexeddbPersistence(projectId, doc);
  persistence.on('synced', () => console.info('content from the database is loaded'));

  function _onClose() {
    state.value = ConnectionState.Disconnected;
    doc.off('update', _yDocUpdateHandler);
    awareness.off('update', _awarenessUpdateHandler);
    websocket = null;
  }

  let reconnectAttemptsLeft = 10;
  function _onError() {
    if (!shouldBeConnected.value) return;
    setTimeout(() => {
      if (reconnectAttemptsLeft > 0) {
        reconnectAttemptsLeft -= 1;
        _connect();
      }
    }, 50);
  }

  function _connect() {
    state.value = ConnectionState.EstablishingConnection;

    // Setup websocket
    websocket = new WebSocket(ySocketUrl(projectId));
    websocket.onmessage = _handleMessage;
    websocket.onclose = _onClose;
    websocket.onerror = _onError;

    // start sending content updates
    doc.on('update', _yDocUpdateHandler);

    // start sending awareness updates
    awareness.on('update', _awarenessUpdateHandler);
  }

  function connect() {
    reconnectAttemptsLeft = 10;
    shouldBeConnected.value = true;
    _connect();
  }

  function disconnect() {
    shouldBeConnected.value = false;
    websocket?.close();
  }

  function send(msg: WSClientMessage, triesLeft = 3) {
    if (!websocket || websocket.readyState !== WebSocket.OPEN) {
      if (!shouldBeConnected.value) {
        console.error(`Websocket intentionally closed. Can't send update`);
        return;
      }
      if (reconnectAttemptsLeft === 0) {
        reconnectAttemptsLeft = 2;
        _connect();
      }
      if (triesLeft > 0) setTimeout(() => send(msg, triesLeft - 1), 100);
      return;
    }

    websocket.send(JSON.stringify(msg));
  }

  const handlers: Map<
    WSServerMessageType,
    Set<(message: unknown) => void>
  > = new Map();

  /// Add a listener, sending updates is paused automatically once disconnected.
  function on<T extends WSServerMessageType>(eventType: T, f: (message: Extract<WSServerMessage, { t: T }>) => void): () => void {
    if (!handlers.has(eventType)) handlers.set(eventType, new Set());
    const set = handlers.get(eventType)!;
    set.add(f as (message: unknown) => void);

    const unsubscribe = () => {
      set.delete(f as (message: unknown) => void);
    };

    return unsubscribe;
  }

  on('u', (msg) => {
    Y.applyUpdate(doc, Uint8Array.fromBase64(msg.update), 'other');
  });

  on('a', (msg) => {
    awarenessProtocol.applyAwarenessUpdate(awareness, Uint8Array.fromBase64(msg.update), 'other');
  });

  function _yDocUpdateHandler(update: Uint8Array, origin: unknown) {
    if (origin === 'other') return;
    send({
      t: "u",
      update: update.toBase64(),
    });
  }

  function _awarenessUpdateHandler({ added, updated, removed }: {
    added: number[],
    updated: number[],
    removed: number[],
  }, _origin: unknown) {
    const changedClients = added.concat(updated).concat(removed);
    const update = awarenessProtocol.encodeAwarenessUpdate(awareness, changedClients);
    send({
      t: 'a',
      update: update.toBase64(),
    });
  }

  function _handleMessage(this: WebSocket, ev: MessageEvent) {
    if (state.value != ConnectionState.Connected) state.value = ConnectionState.Connected;
    const data: WSServerMessage = JSON.parse(ev.data);

    if (!handlers.has(data.t)) return;
    for (const f of handlers.get(data.t)!) {
      f(data);
    }
  }

  if (websocket != null) console.error("BUG: onMounted doesn't work with multiple listeners");
  connect();

  onScopeDispose(() => {
    if (websocket == null) console.error("BUG: onUnmounted doesn't work with multiple listeners");
    disconnect();
  });


  const res = {
    state,
    doc,
    awareness,
    connect,
    disconnect,
    send,
    on,
  };

  if (openSockets.has(projectId)) {
    console.error('Setup concurrency bug.')
  }
  openSockets.set(projectId, res);
  return res;
}

function ySocketUrl(projectId: string, method = "edit") {
  const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${protocol}//${location.host}${API_ROOT}/project/${projectId}/${method}`;
}
