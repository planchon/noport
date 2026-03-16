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

### Customize the subdomain
The subdomain can be changes : 
- `--domain sub_domain` you choose your subdomain 
- `--git-branch` we will infer a sub domain from the branch name 
- `--git-worktree` we will infer a sub domain from the worktree

## Install
The software is not installable yet. will be soon
