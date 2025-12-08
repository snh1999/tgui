# 6. Development Roadmap

### Week 1: Framework Decision + Basic UI
**Goal**: Pick Tauri or Wails, build skeleton

**Tauri Prototype**:
- [ ] Init: `npm create tauri@latest -- --template vue-ts`
- [ ] Create basic UI: Command list (hardcoded data)
- [ ] Test: Spawn simple command (`echo "hello"`)
- [ ] Measure: Build time, bundle size

**Wails Prototype**:
- [ ] Init: `wails init -n commandpal -t vue-ts`
- [ ] Create basic UI: Command list (hardcoded data)
- [ ] Test: Spawn simple command via Go backend
- [ ] Measure: Build time, bundle size, Go learning curve

**Decision Criteria**:
- Which has better process spawning docs?
- Which hot-reload is faster?
- Which feels more natural to you?

**Deliverable**: Document decision in ADR-001, delete losing prototype

---

### Week 2: Core Features
**Goal**: Save/run/view logs for commands

- [ ] **Database Setup**:
  - [ ] SQLite schema (see ADR-003)
  - [ ] CRUD functions: create, read, update, delete commands
  
- [ ] **Command Execution**:
  - [ ] Backend: `execute_command(id)` function
  - [ ] Parse working directory from DB
  - [ ] Spawn process with correct `cwd`
  
- [ ] **Log Viewer**:
  - [ ] Stream stdout/stderr to frontend
  - [ ] Display in separate component
  - [ ] Auto-scroll toggle
  
- [ ] **UI Polish**:
  - [ ] New Command modal
  - [ ] Edit Command modal
  - [ ] Delete confirmation
  - [ ] Search/filter box

**Test**: Can you save your actual dev commands and use them daily?

---

### Week 3: Categories & Favorites
**Goal**: Organize commands, quick access

- [ ] **Categories**:
  - [ ] Category dropdown in New Command modal
  - [ ] Filter by category
  - [ ] Group commands by category in list
  
- [ ] **Favorites**:
  - [ ] Star icon to toggle favorite
  - [ ] Favorites section at top of list
  
- [ ] **Quick Actions**:
  - [ ] Right-click context menu
  - [ ] Keyboard shortcuts (Ctrl+N = New, Ctrl+F = Search)
  
- [ ] **Settings**:
  - [ ] Default terminal shell (bash, zsh, fish)
  - [ ] Log buffer size
  - [ ] Theme (light/dark)

**Test**: Invite a friend to test, watch them use it (don't help!)

---

### Week 4: Templates
**Goal**: Create/apply command templates

- [ ] **Template Management**:
  - [ ] Templates table in DB
  - [ ] Create template from existing commands
  - [ ] Edit/delete templates
  
- [ ] **Apply Template**:
  - [ ] "Create from Template" button
  - [ ] Directory picker
  - [ ] Substitute `{{directory}}` placeholder in commands
  - [ ] Create all commands at once
  
- [ ] **Built-in Templates** (ship with app):
  - [ ] Python Dev (venv, pip, pytest)
  - [ ] Node.js Dev (npm, npx, node)
  - [ ] Docker Common (build, up, down, logs)
  - [ ] Git Workflow (status, add, commit, push)

**Test**: Create a new project, apply template, verify all commands work

---

### Week 5-6: Polish & Distribution
**Goal**: Make it presentable and shareable

- [ ] **UI/UX Polish**:
  - [ ] Icons for categories (🐳 Docker, 🐍 Python, etc.)
  - [ ] Loading states (spinner when running)
  - [ ] Error handling (command not found, directory doesn't exist)
  - [ ] Empty states (no commands yet → helpful onboarding)
  
- [ ] **Export/Import**:
  - [ ] Export commands as JSON
  - [ ] Import commands (merge or replace)
  
- [ ] **Documentation**:
  - [ ] README with screenshots
  - [ ] Quick start guide
  - [ ] GIF demo
  
- [ ] **Packaging**:
  - [ ] Linux: AppImage + .deb
  - [ ] Windows: .msi
  - [ ] macOS: .dmg
  
- [ ] **GitHub**:
  - [ ] Add LICENSE
  - [ ] Create releases with binaries
  - [ ] Add badges (build status, downloads)

**Launch**: Post on r/linux, r/selfhosted, Hacker News


## 6.1 Launch Plan

### 6.1 Pre-Launch (Week 5-6)
- [ ] Create demo GIF
- [ ] Write README with:
  - Problem statement
  - Features list
  - Installation instructions
  - Quick start guide (5 steps to first command)
  - Screenshots
- [ ] Set up GitHub:
  - Tags: `desktop-app`, `command-runner`, `developer-tools`
  - Topics: `tauri` or `wails`, `vue`, `typescript`
- [ ] Build binaries for Linux (priority), Windows, macOS

### 6.2 Launch Day
- [ ] Create GitHub Release v0.1.0
- [ ] Post on Reddit:
  - r/linux (title: "I built a command runner for Linux that saves your frequently used commands")
  - r/commandline
  - r/opensource
  - r/programming (Saturday)
- [ ] Post on Hacker News "Show HN: CommandPal - A desktop app for organizing and running commands"
- [ ] Tweet with #buildinpublic #opensource
- [ ] Post in Discord servers (Tauri/Wails, Vue Land)

### 6.3 Post-Launch (First 2 Weeks)
- [ ] Respond to GitHub issues within 24 hours
- [ ] Fix critical bugs immediately
- [ ] Add "Contributors Welcome" badge if open to PRs
- [ ] Write blog post about building it (good for portfolio)
