import * as Y from 'yjs'
import * as awarenessProtocol from 'y-protocols/awareness.js'
import type { Compiler } from "@/stores/compilerOutput";
import type { ClientMessage, ServerMessage } from '@/api/types';

export class CustomWSProvider {
  constructor(ydoc: Y.Doc,
              wsUri: string,
              onLogUpdate: (uuid: string, log: string) => void,
              onPdfUpdate: (uuid: string, pdfSrc: string | null) => void,
  ) {
    this.closed = false;
    this.websocket = null;

    this.ydoc = ydoc;
    this.ydoc.on('update', this.#handleUpdate.bind(this));

    this.#connect(wsUri, onLogUpdate, onPdfUpdate);

    this.awareness = new awarenessProtocol.Awareness(ydoc);
    this.awareness.on('update', ({ added, updated, removed }: { added: number[]; updated: number[]; removed: number[] }, origin: unknown) => {
      if (origin == 'other') return;
      const changedClients = added.concat(updated).concat(removed);
      const msg = awarenessProtocol.encodeAwarenessUpdate(this.awareness, changedClients);
      this.#send({
        t: "a",
        update: msg.toBase64(),
      });
    })
  }

  private websocket: WebSocket | null;

  private closed: boolean;

  private readonly ydoc: Y.Doc;

  awareness: awarenessProtocol.Awareness;

  #handleUpdate(update: Uint8Array, origin: unknown) {
    if (origin === 'other') return;
    this.#send({
      t: "u",
      update: update.toBase64(),
    });
  }

  #send(msg: ClientMessage, triesLeft = 3) {
    if (!this.websocket || this.websocket.readyState !== WebSocket.OPEN) {
      console.error(`Websocket closed. Can't send update`);
      if (triesLeft > 0) setTimeout(() => this.#send(msg, triesLeft - 1), 100);
      return;
    }

    this.websocket.send(JSON.stringify(msg));
  }

  #connect(wsUri: string,
           onLogUpdate: (uuid: string, log: string) => void,
           onPdfUpdate: (uuid: string, pdfSrc: string | null) => void,
       ): void {
    this.websocket = new WebSocket(wsUri);
    this.websocket.addEventListener("close", () => {
      if (this.closed) return;
      console.log("Socket closed: reconnecting...")
      setTimeout(() => this.#connect(wsUri, onLogUpdate, onPdfUpdate), 100);
    });
    this.websocket.addEventListener("message", async e => {
      const msg = JSON.parse(e.data) as ServerMessage;
      switch (msg.t) {
        case "u":
          Y.applyUpdate(this.ydoc, Uint8Array.fromBase64(msg.update), 'other');
          break;

        case "a":
          awarenessProtocol.applyAwarenessUpdate(this.awareness, Uint8Array.fromBase64(msg.update), 'other');
          break;

        case "setLog":
          onLogUpdate(msg.uuid, msg.data);
          break;

        case "setPdf":
          if (msg.content) {
            onPdfUpdate(msg.uuid, msg.content);
          } else {
            onPdfUpdate(msg.uuid, null);
          }
          break;

        case "assetUpdated":
          console.log("Ignoring assetUpdated message")
          break;

          default:
            console.error(`Unexpected data type ${typeof e.data}`);
      }
    });
  }

  dispose() {
    if (this.closed) return;
    this.closed = true;
    this.ydoc.off("update", this.#handleUpdate.bind(this));
    if (this.websocket) {
      this.websocket.close();
      this.websocket = null;
    }
  }

  startCompile(uuid: string, compiler: Compiler) {
    if (!this.websocket) {
      console.error(`Websocket closed. Can't send compile command`);
      return;
    }
    if (this.websocket.readyState === WebSocket.CLOSED
      || this.websocket.readyState === WebSocket.CLOSING) {}
    this.#send({
      t: "compile",
      uuid: uuid,
      compiler: compiler
    });
  }

  setAsset(uuid: string, content: Uint8Array) {
    if (!this.websocket) {
      console.error(`Websocket closed. Can't upload asset`);
      return;
    }
    this.#send({
      t: "setAsset",
      uuid: uuid,
      data: content.toBase64(),
    });
  }
}