# Requirements Specification: TGUI v0.1.0

## 2.1 Functional Requirements

| ID    | Requirement                            | Priority | Target                                                                      |
| ----- | -------------------------------------- | -------- | --------------------------------------------------------------------------- |
| FR-01 | Command Library                        | Must     | Save commands with name, description, category, star favorites              |
| FR-02 | Execution Context                      | Must     | Run command in specific directory                                           |
| FR-03 | Log Viewer                             | Must     | Real-time stdout/stderr streaming                                           |
| FR-04 | Quick Search/Filtering                 | Must     | Filter commands by name/category                                            |
| FR-05 | Templates and copying                  | Must     | Create command sets (with context, commands)                                |
| FR-06 | Shell command execution with arguments | Must     | Arguments parsing, Process starts, PID returned, logs stream in 1s          |
| FR-07 | Kill processes/halt command execution  | Must     | both graceful and forced kill works, UI updates in <500ms                   |
| FR-08 | Background process                     | Must     | Close window → process keeps running → reopen window → logs still streaming |
| FR-09 | Environment Variables                  | Should   | Pass env vars with commands (e.g., `API_KEY=xxx`)                           |
| FR-10 | Command Chaining                       | Should   | Run sequence of commands                                                    |
| FR-11 | System tray icon                       | Should   | Tray icon updates on spawn/kill, hover shows count                          |
| FR-12 | Variables in Commands                  | Should   | `git commit -m "$message"` with prompt                                      |
| FR-13 | Multi-window Logs                      | Could    | Can drag log window to second screen, close main window, logs persist       |
| FR-14 | Process group management               | Could    | eg- kill entire process tree                                                |
| FR-15 | Search/filter logs by regex            | Could    | Ctrl+F opens search bar, filters in <500ms on 10k lines                     |
| FR-16 | Export/Import                          | Could    | Share templates as JSON                                                     |
| FR-17 | Schedule Commands                      | Could    | Cron-like scheduling                                                        |
| FR-18 | Command History                        | Could    | Track execution history per command                                         |

## 2.2 Non-Functional Requirements

| ID     | Requirement               | Target                                              |
| ------ | ------------------------- | --------------------------------------------------- |
| NFR-01 | **RAM usage**             | ≤ 50 MB Idle                                        |
| NFR-02 | **Startup time**          | Cold start < 3s, warm start < 500ms                 |
| NFR-03 | **Processe count**        | Handle 10-20 concurrent background processes        |
| NFR-04 | **Log buffer size**       | Keep last 10,000 lines per process in memory        |
| NFR-05 | **Compatibility**         | Ubuntu 22.04+, Fedora 42+, Arch, Windows 11+, macOS |
| NFR-06 | **Build size**            | Final Linux AppImage < 20 MB                        |
| NFR-07 | **Process spawn latency** | < 500 ms                                            |
