# Project Charter: **TGUI** 

## 1.1 Project Identity

- **Project Name**: TGUI
- **Tagline**: "Terminal commands in one click"
- **Codename**: "pancake"  
- **Version**: 0.1.0 (MVP)
- **License**: [GPL-3.0]
- **Repository**: https://github.com/snh1999/tgui
- **Start Date**: [05-12-2025]
- **Target MVP**: 4-6 weeks
  
## 1.2 Problem Statement
**Pain Point**:
Existing command runners are usually either 
- Terminal applications with command persistance ability 
- Application launcher with customization options
which causes limitation leading to

1. Terminal tabs don't survive window closure—lose logs when you close terminal
2. No easy way to monitor logs from multiple long-running commands
3. No reproducible template for repetitive commands, or proper persistance with one click command execution ability


**Solution and Priorities**:
A cross-platform desktop app that:
- Stores commands with their execution context (directory, environment)
- Organizes/filters commands by categories/criteria 
- Shows real-time logs in dedicated viewer
- One-click execution from a searchable list
- Templates for common workflows (e.g., "Python Dev" template with venv commands)
- Survives main window closure via system tray (background task processing)
  

## 1.3 Success Metrics (MVP)

| Metric | Target | Why It Matters |
|--------|--------|----------------|
| **Save & Execute** | < 3-5 seconds from app open to command running | Core workflow must be fast |
| **Command search** | < 200ms for 100+ commands | Users will have many commands |
| **Log streaming** | < 200ms latency | Debugging needs real-time feedback |
| **User adoption** | 50 users in first 3 months | Validates real need |
| **RAM usage** | ≤ 40 MB Idle | App needs to be optimized as utility app |
| **Max concurrent processes** | 10-20 processes initially | Blocks premature optimization |
| **Distribution** | 3+ package formats | Successfully install via exe(windows), flatpak(Linux), AppImage (Linux) and beyond |

## 1.4 Value Proposition
**Why this vs alternatives?**
- **vs Terminal aliases**: Portable, visual, organized by project
- **vs launcher apps with commands**: Integrated execution + logs
- **vs Shell history (Ctrl+R)**: Categorized, persistent, context-aware

**User base**
- **Target User**: Regular users not wanting to open terminal Interface
- **Primary User**: Developers/enthusiasts looking for conveniance
