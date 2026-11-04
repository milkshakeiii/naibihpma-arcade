# Builds the committed game on the cloud server and restarts it there, saving its world first.
#   powershell -ExecutionPolicy Bypass -File tools\server\deploy.ps1
# Only what's committed goes: the source of HEAD, as Git stores it.
param(
    [string]$Server = 'monster-workshop',
    [string]$Key = "$env:USERPROFILE\.ssh\monsterworkshop_ed25519"
)
$ErrorActionPreference = 'Stop'

$hcloud = (Get-Command hcloud -ErrorAction SilentlyContinue).Source
if (-not $hcloud) {
    # A shell older than hcloud's install doesn't have winget's PATH change yet.
    $hcloud = Get-ChildItem "$env:LOCALAPPDATA\Microsoft\WinGet\Packages\HetznerCloud.CLI_*" -Recurse -Filter hcloud.exe |
        Select-Object -First 1 -ExpandProperty FullName
}
$ip = (& $hcloud server ip $Server).Trim()
$repo = Split-Path (Split-Path (Split-Path $PSScriptRoot))

$head = git -C $repo log -1 --format='%h %s'
"Deploying $head to $Server ($ip)."
if (git -C $repo status --porcelain -- monsterworkshop) { 'Uncommitted changes stay here.' }

$tar = Join-Path $env:TEMP 'mw.tar.gz'
$sh = Join-Path $env:TEMP 'remote.sh'
git -C $repo -c core.autocrlf=false archive --format=tar.gz -o $tar HEAD monsterworkshop
[IO.File]::WriteAllText($sh, ((Get-Content -Raw "$PSScriptRoot\remote.sh") -replace "`r`n", "`n"))

$opts = @('-i', $Key, '-o', 'StrictHostKeyChecking=accept-new', '-o', 'BatchMode=yes')
scp -q @opts $tar $sh "root@${ip}:/tmp/"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
ssh @opts "root@$ip" 'bash /tmp/remote.sh'
exit $LASTEXITCODE
