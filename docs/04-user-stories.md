# 3. User Stories

Trello Link: https://trello.com/b/DhteKuWv/tgui

### TGUI-01: Save and Execute Command with Context (FR-01, FR-02, FR-06)
**As a** developer working on multiple projects  
**I want to** save a command with its execution directory and arguments  
**So that** I can run it correctly without remembering the context

**Acceptance Criteria**:
1. Click "New Command" button
2. Fill form:
   - Name: "Start Dev Server"
   - Command: `docker-compose`
   - Arguments: `up -d --build`
   - Directory: `/home/user/projects/myapp`
   - Category: "Docker"
3. Click Save → Command appears in list (FR-01)
4. Click [▶️ Run] → Process spawns in correct directory (FR-02)
5. PID returned within 500ms (NFR-07)
6. Logs start streaming within 1 second (FR-06)
7. Verify in terminal: `ps aux | grep docker-compose` shows process with correct working directory

**Definition of Done**:
- [ ] Command saved to SQLite database
- [ ] Arguments parsed correctly (handles spaces, quotes)
- [ ] Process spawns with correct `cwd`
- [ ] UI shows "Running" status with PID
- [ ] Logs stream to frontend in <1s

---

### TGUI-02: Real-Time Log Monitoring (FR-03, FR-08)
**As a** developer debugging a build process  
**I want to** see real-time logs and keep them even if I close the window  
**So that** I don't lose important output

**Acceptance Criteria**:
1. Run command: `npm run build:watch`
2. Log viewer appears in main window
3. New log lines appear within 100ms (FR-03)
4. Color coding: stdout (white), stderr (red)
5. Auto-scroll enabled by default
6. Close main window → process keeps running (FR-08)
7. Verify with: `ps aux | grep npm` → process still alive
8. Reopen window → logs show full history from start
9. New logs continue streaming in real-time

**Definition of Done**:
- [ ] Logs stream with <100ms latency
- [ ] Process survives window closure (tested with `ps aux`)
- [ ] Log buffer maintains 10,000 lines (NFR-04)
- [ ] Reopen window shows historical + live logs
- [ ] Auto-scroll stops when user scrolls up, resumes at bottom

---

### TGUI-03: Stop Running Process (FR-07)
**As a** developer  
**I want to** stop a long-running command gracefully or forcefully  
**So that** I can terminate processes cleanly or force-kill if hung

**Acceptance Criteria**:
1. Run command: `python -m http.server 8000`
2. Process shows "Running" status with PID
3. Click "Stop" button → sends SIGTERM (FR-07)
4. UI updates to "Stopping..." within 500ms (NFR)
5. Process exits → UI shows "Stopped" with exit code
6. Right-click → "Force Kill" → sends SIGKILL
7. Hung process (e.g., `sleep 1000`) terminates immediately
8. Verify: `ps aux | grep python` shows no process

**Definition of Done**:
- [ ] SIGTERM implementation working
- [ ] SIGKILL (force kill) working
- [ ] UI updates status in <500ms
- [ ] Exit code displayed in UI
- [ ] No zombie processes (`ps aux | grep defunct`)

---

### TGUI-04: Quick Access with Search and Favorites (FR-01, FR-04)
**As a** developer with 50+ saved commands  
**I want to** quickly find and star my most-used commands  
**So that** I can access them instantly

**Acceptance Criteria**:
1. Save 50 commands across 5 categories
2. Star 5 commands as favorites (FR-01)
3. Favorites appear in dedicated section at top
4. Type "docker" in search box (FR-04)
5. List filters to show only matching commands in <200ms
6. Search matches name, command, description
7. Clear search → all commands visible again
8. Click category filter → shows only that category

**Definition of Done**:
- [ ] Star/unstar toggle working
- [ ] Favorites section always visible at top
- [ ] Search filters in <200ms on 100+ commands
- [ ] Search highlights matched text
- [ ] Category filter works with search

---

### TGUI-05: Command Templates (FR-05)
**As a** Python developer starting a new project  
**I want to** apply a "Python Dev" template  
**So that** I get all common commands instantly configured

**Acceptance Criteria**:
1. Navigate to Templates section
2. Select "Python Development" template
3. Template shows preview:
   - Create virtual environment
   - Activate venv
   - Install dependencies
   - Run tests
4. Click "Apply Template"
5. Directory picker opens → select `/home/user/projects/new-ml-project`
6. Template creates 4 commands with correct directory (FR-05)
7. All commands appear in "Python" category
8. Each command has correct working directory set
9. Test: Run "Activate venv" → works in selected directory

**Definition of Done**:
- [ ] Template stored in database
- [ ] Directory picker integrated
- [ ] All template commands created with substituted directory
- [ ] User can copy existing command to create similar ones
- [ ] Built-in templates: Python, Node.js, Docker, Git

---

### TGUI-06: Background Process (FR-08, FR-11)
**As a** developer running a local LLM/dev server  
**I want to** keep the server running when I close TGUI  
**So that** I can use the LLM from other applications

**Acceptance Criteria**:
1. Run command: `ollama serve` (long-running LLM server)
2. Process shows "Running" in UI
3. Close main window completely (FR-08)
4. System tray icon remains visible (FR-11)
5. Hover tray icon → tooltip shows "1 process running"
6. Verify: `ps aux | grep ollama` → process alive
7. Wait 2 minutes (simulate background operation)
8. Click tray icon → window reopens
9. Command still shows "Running" with live logs
10. LLM server still responding to requests

**Definition of Done**:
- [ ] Process survives window closure (no parent process dependency)
- [ ] Tray icon shows running process count
- [ ] Tray icon updates on spawn/kill events
- [ ] Logs persist and continue streaming after reopen
- [ ] Test with actual long-running process (10+ minutes)

---

### TGUI-07: Environment Variables for Commands (FR-09)
**As a** developer (User) running scripts that need API keys  
**I want to** save environment variables with my commands  
**So that** I don't have to export them manually each time

**Acceptance Criteria**:
1. Create new command: "Deploy to Production"
2. Command: `./deploy.sh`
3. Add environment variables (FR-09):
   - `API_KEY=sk-test-abc123`
   - `ENV=production`
   - `DEBUG=false`
4. Save command
5. Run command → verify env vars passed
6. Check logs → script receives correct environment variables
7. Edit command → modify env var value → run → new value used

**Definition of Done**:
- [ ] UI for adding key-value pairs
- [ ] Env vars stored in database (encrypted? or warn user)
- [ ] Process spawned with correct environment
- [ ] Can edit/delete individual env vars
- [ ] Warning displayed: "Env vars stored in plain text"

---

### TGUI-08: Batch Command Execution (FR-10)
**As a** developer (user) with complex build workflows or a linux user trying to update the system   
**I want to** run a sequence of commands automatically  
**So that** I don't have to manually trigger each step

**Acceptance Criteria**:
1. Create command chain: "Full Build Pipeline" (FR-10)
2. Add commands in sequence:
   - `git pull origin main`
   - `npm install`
   - `npm run build`
3. Click "Run Chain"
4. First command executes → waits for completion
5. If exit code = 0 → next command starts automatically
6. If any command fails → chain stops, shows error
7. Logs show all commands with timestamps
8. User can stop chain mid-execution

**Definition of Done**:
- [ ] UI to define command sequences
- [ ] Sequential execution logic (wait for exit)
- [ ] Error handling (stop on failure)
- [ ] Option: "Continue on error" checkbox
- [ ] Logs clearly show which command is running

---

### TGUI-09: Multi-Window Log Viewing (FR-13)
**As a** developer with multiple monitors  
**I want to** open logs in separate windows  
**So that** I can monitor multiple commands simultaneously

**Acceptance Criteria**:
1. Run 3 commands: Server, Ollama, Docker
2. Right-click server command → "Open in New Window" (FR-13)
3. New window opens with server logs only
4. Drag window to second monitor
5. Repeat for Ollama and Docker
6. Close main window → all log windows stay open
7. All logs continue updating in real-time
8. Close any log window → process keeps running
9. Click "Stop" in log window → process terminates

**Definition of Done**:
- [ ] `open_log_window(command_id)` API implemented
- [ ] Each log window has independent lifecycle
- [ ] Logs stream to correct window (filtered by PID)
- [ ] Window title shows command name
- [ ] Stop button in each log window

---

### TGUI-10: Process Organizing (FR-14)
**As a** developer running npm scripts  
**I want to** kill all child processes when stopping  
**So that** webpack/node children don't linger

**Acceptance Criteria**:
1. Run command: `npm run dev` (spawns webpack child)
2. Check with `pstree -p $(pgrep npm)` → see child PIDs
3. Enable option: "Kill process tree" (FR-14)
4. Click "Stop"
5. All processes terminate (parent + children)
6. Verify: `pstree` shows nothing, no zombie processes

**Definition of Done**:
- [ ] Process group ID (PGID) tracked on spawn
- [ ] `kill_process_group()` sends signal to entire group
- [ ] UI checkbox: "Kill process tree"
- [ ] Tested with: npm, docker-compose, python multiprocessing

---

### TGUI-11: Search Logs (FR-15)
**As a** developer debugging errors in logs  
**I want to** search and filter logs by regex  
**So that** I can find specific error messages quickly

**Acceptance Criteria**:
1. Run command with upto 10,000+ log lines (maximum count)
2. Press Ctrl+F → search bar appears (FR-15)
3. Type regex: ERROR|WARN
4. Matching lines highlighted in <500ms
5. Toggle "Filter mode" → only shows matching lines
6. Search works on live streaming logs
7. Navigate: F3/Ctrl + n (next match), Shift+F3/Ctrl + Shift + n (previous)

**Definition of Done**:
- [ ] Search bar with regex support
- [ ] Highlight mode vs filter mode
- [ ] Performance: <500ms on 10k lines
- [ ] Works with live streaming logs
- [ ] Keyboard shortcuts implemented

---

### TGUI-12: Export and Share (FR-16)
**As a** User  
**I want to** export my command templates as JSON  
**So that** my team can import and use the same workflows  

**Acceptance Criteria**:
1. Create template: "Test template" with multiple commands
2. Click "Export Template" _**(FR-16)**_ and optionally add config info
3. Save as `test-template.json`
4. Teammate opens TGUI → "Import Template"
5. Select `test-template.json`
6. Preview shows all commands
7. Select directory and other variables set in template
8. Click "Import" → template available in their app
9. Apply template → commands work correctly

**Definition of Done**:
- [ ] Export to JSON format
- [ ] Import with validation (schema check)
- [ ] Preview before import
- [ ] Merge vs replace option
- [ ] Include: commands, categories, env vars
