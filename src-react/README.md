# Tauri + React + Typescript

This template should help get you started developing with Tauri, React and Typescript in Vite.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Mobile development

### Windows Firewall

On Windows, the firewall cound be an issue. Try this:

**Quick test first — temporarily disable firewall**
If your phone can connect after disabling it, the firewall is confirmed as the cause.

**Proper fix — add a firewall rule (run PowerShell as Admin):**
```powershell
New-NetFirewallRule -DisplayName "Dev Server 1420" -Direction Inbound -Protocol TCP -LocalPort 1420 -Action Allow
```

**Or via GUI:**
1. Search → "Windows Defender Firewall with Advanced Security"
2. Inbound Rules → New Rule
3. Port → TCP → 1420 → Allow → All profiles → Save

**Also check if Windows marked your network as Public** (Public networks block most inbound connections):

```powershell
# Check current network profile
Get-NetConnectionProfile
```
If `NetworkCategory` shows `Public`, change it to `Private`:
```powershell
Set-NetConnectionProfile -Name "YOUR_NETWORK_NAME" -NetworkCategory Private
```

**Verify the port is actually listening on 0.0.0.0:**
```powershell
netstat -an | findstr 1420
```
You want to see `0.0.0.0:1420` — if it shows `127.0.0.1:1420` the server binding is still the problem.
