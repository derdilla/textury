# Editing a projects

Projects are stored in a [CRDT](https://jakelazaroff.com/words/an-interactive-intro-to-crdts/) for a number of reasons:
- the same file can be concurrently edited
- battle-tested to be resilient against most forms of accidental data loss
- history is build-in

The downside is, that the client can't assume it have the most up-to-date view on the data. Itself or a colaborator could have made unsynced changes.

Synchronization happens through a custom protocol with the server. Messages are send over websocket since the protocol is bidirectional. We don't use the existing y-websocket since it didn't make sense reimplementing the server in rust and we want to send custom message via the same websocket.


## Protocol details

The most important kinds messages the client can receive are:
- **awareness** (`a`): Joined users or cursor positions change.
- **update** (`u`): Partial or full updates to the CRDT.
- Compilation log, PDF or asset updates

The clients can also send *awareness* and *update* messages and can trigger compilations (`compile`).

On connection the server sends the latest version of the document. Subsequent updates from other clients are proxied.

You can find implementation details in `frontend/src/composables/editorWS.ts` and `server/src/websocket.rs`.


## CRDT structure

The CRDT is more than a single text file. At a top level it contains the `files` property which holds an array of all stored text files, assets and directories. To allow safely evolving the format each entry contains a version field. The files array is entirely flat and does not contain the conventional file tree. 

Each _entry_ of the files array is identified by a UUID set at creation and never modified. This brings a persistent handle across renames. The `type` field also shouldn't be changed. It differentiates text-files, binary-files, and directories.

For interoperability with the familiar file tree and the LaTeX/Typst compilers, entries contain a field with the full path. This path is an editable CRDT, so a concurrent move and rename don't override each other. When writing this array to a tree structure implementations need create directories for every path component, even if no _directory_ entry exists. You can think of explicit directory objects as placeholders for path we want to synchronize but not create a file in.

Text files contain an additional text attribute with the document content. Naturally this is also a CRDT. Asset files are stored in their own database rows by UUID, and updated and retrieved via REST and websocket API.

A good reference for the available fields is at `frontend/src/utils/crdtFilesEntry.ts`. Deviations in other implementations should be considered bugs. A good client is implemented at `frontend/src/composables/editorFileTree.ts`.

## Further reading

- yjs docs: https://docs.yjs.dev/
- normal y-websocket implementation: https://github.com/yjs/y-websocket/blob/master/src/y-websocket.js
