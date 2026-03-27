# noport 
Remove all the port from your life

```diff
- "dev": "vite"        # http://localhost:5173
+ "dev": "noport vite" # https://app.local
```

## Usage
Add noport before your app, we will infer a cool, fixed subdomain, using HTTPS : 

```bash
# Served on http://localhost:5173
vite        

# Served on http://myapp.local:2828
noport -- vite 
```

> [!NOTE]
> NoPort daemon can run non-root, but need to run as root if you want to use `port < 1024` or use a TLD different than `.localhost`. 

### Commands

- `noport -- anything` to start a process through noport 
- `noport start` to start the daemon (and the proxy)

#### Other commands
- `noport stop` to stop the daemon
- `noport status` to get the status of daemon

## Roadmap
before calling `noport` ready, I would like to ship these features (this is the `v1` roadmap)

- [x] socket / client communication
- [x] sudo management 
- [x] port < 1024 management (port 80)
- [ ] custom `tld` (like `.lan`, `.home`, `.test` etc)
- [ ] automatic sub-domain generation (based on folder, git branch, git worktree)
- [ ] `https` support
- [ ] support famous frameworks (vite, next, nest, ...)

## Install
The software is not installable yet. will be soon (when the roadmap of the v1 is finished)
