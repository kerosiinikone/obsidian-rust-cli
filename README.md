# obsidian-rust-cli

**Quick capture from terminal**: create a new "idea" note / append to daily notes from a "fleeting thought". Open, create (daily) notes from terminal, pretty printing. **Incorporate this**: "Vault Statistics Dashboard"

### Workflow (for self)

1. Setup the repo / vault (_config file_)
   - Read config file into memory before exec (cached?)
   - Commands to add / edit config from terminal
   - Give cfg path directly as a **flag**
2. Commands to create, append, display (_pretty_) and open (_daily_) notes
   - ...

### Commands (Windows PS)

```powershell
$Env:VAULT_PATH="env_var"; .\target\debug\obsidian-rust-cli.exe new Hello
.\target\debug\obsidian-rust-cli.exe --vault "vault" append -n "note" Hello
```

### TODOs

Licence
Scripts
Makefile
Signal processing
