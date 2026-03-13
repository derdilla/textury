## Structure

The _server_ handles login, socials and loading projects. One _worker_ is loaded per open project into a sandbox and handles file CRUD, building, and git.


:eyes:
- https://www.swiftlatex.com/
- https://demos.yjs.dev/monaco/monaco.html

## build
- in frontend: `bun run build-only`
- build server

## Design goals

- Keep the server minimal
- Put nothing on the worker, users shouldn't have access to if they get a root shell
- Be resource efficient on idle
