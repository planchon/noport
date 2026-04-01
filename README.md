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

# Served on https://myapp.localhost 
noport -- vite 
```

> [!NOTE]
> NoPort daemon can run non-root, but need to run as root if you want to use `port < 1024` or use a TLD different than `.localhost`. 

### Launching the daemon
The daemon is the process serving all the proxy requests from your browser to the child process.

```bash
# Start the daemon and change the TLD to .lan 
noport start --tld lan

# Start the daemon and change the port to 8080
noport start --port 8080

# Start the daemon in the foreground  
noport start --foreground
```

## Commands

- `noport -- anything` to start a process through noport 
- `noport start` to start the daemon 
- `noport setup` generate the CA root certificate (and put it in `~/.noport/certs/ca.pem`) 
- `noport trust` trust the CA root certificate (requires `sudo`)

### Arguments
Before the `--` argument, you can use the following arguments:
- `--domain` to change the subdomain used by the proxy
- `--app-port` to change the port of the child process (your app)

For the **start** command:
- `--foreground` to run the daemon in the foreground
- `--port` to change the port used by the proxy
- `--tld` to change the TLD (default is `.localhost`) (requires `sudo`)
- `--https` to use HTTPS (default is HTTP) (requires `sudo`)

## Roadmap
before calling `noport` ready, I would like to ship these features (this is the `v1` roadmap)

- [x] socket / client communication
- [x] sudo management 
- [x] port < 1024 management (port 80)
- [x] custom `tld` (like `.lan`, `.home`, `.test` etc)
- [x] `https` and `wss` support
- [ ] automatic sub-domain generation (based on folder, git branch, git worktree)
- [ ] support famous frameworks (vite, next, nest, ...)
- [ ] CI / Release process
- [ ] Usage doc and Architecture doc

### To improve (not for the v1, for later)
- [ ] Better error boundary between proxy and daemon
- [ ] Stop buffering the request in the proxy -> stream them
- [ ] Async openssl operations 
- [ ] Cleanup lifecycle (socket, hosts)
- [ ] Default pages (no proxy found, error in the proxy)
- [ ] Better OS handling in the code, have a clean architecture

## Install
The software is not installable yet. will be soon (when the roadmap of the v1 is finished)
