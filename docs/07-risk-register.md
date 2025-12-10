# 07. Risk Register

**Last Updated**: 2025-12-10  
**Review Schedule**: Bi-Weekly during development  

| ID/Status | Risk | Probability/Impact | Mitigation | 
|------------|------|-------------------|-----------|
| R-01 (Active) | Tauri Reload takes too long | Medium/High | If tauri too slow (frontend-only dev mode for UI work), consider Wails |
| R-02 (Monitoring) | Process spawning bugs | High/High | Test thoroughly on Linux, Windows, macOS |
| R-03 (Monitoring) | SQLite errors with concurrent reads/writes | Medium/Medium | Use WAL mode; single-writer architecture; test with 10+ concurrent commands |
| R-04 (Active) | **Cross-platform path handling breaks** (Windows `\` vs Unix `/`) | High/High | Use framework's path utilities; test on all 3 platforms in CI; normalize all paths on save | 
| R-05 (Monitoring) | **Process leaks if app crashes** (orphaned processes) | Low/High | Use process groups (setpgid); register signal handlers; document manual cleanup (`pkill -f tgui`) | 
| R-06 (Monitoring) | **Database corruption** from improper shutdown | Low/High | Enable SQLite journal; atomic writes; backup before schema migrations; test crash recovery |
| R-07 (Monitoring) | **Log streaming lags at 1000+ lines/sec** | Medium/Medium | Use virtual scrolling; render only visible lines; batch log events (send every 100ms instead of per-line) |  |
| R-08 (Monitoring) | **System tray doesn't work on Wayland** (Linux) | Medium/Medium | Test on Ubuntu 24.04 (Wayland); fallback to taskbar icon if tray unavailable | 


**Status Key**:
- **Active**: Currently being mitigated
- **Monitoring**: Watching, no action yet
- **Occurred**: Risk happened, executing contingency
- **Closed**: Risk resolved or no longer relevant

---

## Risk Deep Dives

### R-01: Hot Reload Time (Tauri Rust Compile)
**Contingency Plan**:
- **Primary**: Use `npm run dev` (frontend-only) for UI work → instant hot reload
- **Secondary**: Use `cargo watch -x run` for incremental backend builds (~30s vs 2min)
- **Batch backend changes**: Group Rust changes together, test frontend first
- **If unbearable**: Consider prototyping backend logic in TypeScript first, port to Rust later

**Early Warning Signs**:
- Spending >30 min/day waiting for compiles
- Avoiding backend changes due to compile friction
- Frustration level increasing
