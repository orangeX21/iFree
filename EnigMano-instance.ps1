$ErrorActionPreference = "Stop"

function Timestamp { (Get-Date).ToString("yyyy-MM-dd HH:mm:ss") }
function Log($msg) { Write-Host "[ENIGMANO $(Timestamp)] $msg" }
function Fail($msg) { Write-Error "[ENIGMANO-ERROR $(Timestamp)] $msg"; Exit 1 }

# === ASCII BANNER ===
$now = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
Write-Host @"
----------------------------------------------------
         ENIGMANO INSTANCE $env:INSTANCE_ID BOOTING
----------------------------------------------------
  STATUS    : Initializing deployment sequence
  TIME      : $now
  ARCHITECT : SHAHZAIB-YT
----------------------------------------------------
"@

# === ENVIRONMENT VARIABLES ===
$SECRET_SHAHZAIB = $env:SECRET_SHAHZAIB
$NGROK_SHAHZAIB  = $env:NGROK_SHAHZAIB
$INSTANCE_ID     = [int]$env:INSTANCE_ID
$NEXT_INSTANCE_ID = $INSTANCE_ID + 1
$WORKFLOW_FILE   = "enigmano.yml"
$BRANCH          = "main"
$RUNNER_ENV      = $env:RUNNER_ENV

# === TUNNEL SETUP ===
Remove-Item -Force .\ngrok.exe, .\ngrok.zip -ErrorAction SilentlyContinue
Invoke-WebRequest https://bin.equinox.io/c/bNyj1mQVY4c/ngrok-v3-stable-windows-amd64.zip -OutFile ngrok.zip
Expand-Archive ngrok.zip -DestinationPath .
.\ngrok.exe authtoken $NGROK_SHAHZAIB
Log "Secure transport channel initiated"

# === ACCESS ENABLEMENT ===
Set-ItemProperty -Path "HKLM:\System\CurrentControlSet\Control\Terminal Server" -Name "fDenyTSConnections" -Value 0
Enable-NetFirewallRule -DisplayGroup "Remote Desktop"
Set-ItemProperty -Path "HKLM:\System\CurrentControlSet\Control\Terminal Server\WinStations\RDP-Tcp" -Name "UserAuthentication" -Value 1
$secPass = ConvertTo-SecureString "P@ssw0rd!" -AsPlainText -Force
Set-LocalUser -Name "runneradmin" -Password $secPass
Log "Instance ingress protocols activated"

# === REGION SCAN LOOP ===
$tunnel = $null
$regionList = @("us", "eu", "ap", "au", "sa", "jp", "in")
$regionIndex = 0

while (-not $tunnel) {
    $region = $regionList[$regionIndex]
    $regionIndex = ($regionIndex + 1) % $regionList.Count
    Log "Scanning operational sector: $region"

    Get-Process ngrok -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Process -FilePath .\ngrok.exe -ArgumentList "tcp --region $region 3389" -WindowStyle Hidden
    Start-Sleep -Seconds 10

    try {
        $resp = Invoke-RestMethod -Uri "http://127.0.0.1:4040/api/tunnels"
        $tunnel = ($resp.tunnels | Where-Object { $_.proto -eq "tcp" }).public_url
        if ($tunnel) { break }
    } catch {
        Log "Sector scan failed, shifting to next zone..."
    }

    Start-Sleep -Seconds 5
}

# === DATA VAULT CREATION ===
try {
    $desktopPath = [Environment]::GetFolderPath("Desktop")
    $dataFolderPath = Join-Path $desktopPath "Data"
    if (-not (Test-Path $dataFolderPath)) {
        New-Item -Path $dataFolderPath -ItemType Directory | Out-Null
        Log "Data vault established at $dataFolderPath"
    } else {
        Log "Data vault already exists at $dataFolderPath"
    }
} catch {
    Fail "Failed to create data vault: $_"
}

# === OUTPUT CREDENTIAL NOTICE ===
$tunnelClean = $tunnel -replace "^tcp://", ""
Write-Host "::notice title=EnigMano Link::Host: $tunnelClean`nUser: runneradmin`nPass: P@ssw0rd!"

# === TIMERS ===
$totalMinutes    = 340
$handoffMinutes  = 330
$shutdownMinutes = 335
$startTime       = Get-Date
$endTime         = $startTime.AddMinutes($totalMinutes)
$handoffTime     = $startTime.AddMinutes($handoffMinutes)
$shutdownTime    = $startTime.AddMinutes($shutdownMinutes)

# === HANDOFF MONITOR (randomized) ===
while ((Get-Date) -lt $handoffTime) {
    $now       = Get-Date
    $elapsed   = [math]::Round(($now - $startTime).TotalMinutes, 1)
    $remaining = [math]::Round(($endTime - $now).TotalMinutes, 1)
    Log "Instance uptime: $elapsed min | Mission window remaining: $remaining min"

    $waitMinutes = Get-Random -Minimum 15 -Maximum 30
    Start-Sleep -Seconds ($waitMinutes * 60)
}

# === DISPATCH NEXT INSTANCE ===
try {
    $userInfo = Invoke-RestMethod -Uri "https://api.github.com/user" -Headers @{ Authorization = "Bearer $SECRET_SHAHZAIB" }
    Log "Authenticated controller: $($userInfo.login)"

    if ($env:GITHUB_REPOSITORY) {
        $userRepo = $env:GITHUB_REPOSITORY
        Log "Detected repository from environment: $userRepo"
    } else {
        $repos = Invoke-RestMethod -Uri "https://api.github.com/user/repos?per_page=100" -Headers @{ Authorization = "Bearer $SECRET_SHAHZAIB" }
        $currentRepo = (Get-Location).Path | Split-Path -Leaf
        $userRepo = ($repos | Where-Object { $_.name -eq $currentRepo }).full_name
        if (-not $userRepo) { Fail "Repository '$currentRepo' not found in your account." }
    }

    $dispatchPayload = @{
        ref    = $BRANCH
        inputs = @{ INSTANCE = "$NEXT_INSTANCE_ID" }
    } | ConvertTo-Json -Depth 3

    $dispatchURL = "https://api.github.com/repos/$userRepo/actions/workflows/$WORKFLOW_FILE/dispatches"

    Invoke-RestMethod -Uri $dispatchURL -Headers @{
        Authorization = "Bearer $SECRET_SHAHZAIB"
        Accept        = "application/vnd.github.v3+json"
    } -Method Post -Body $dispatchPayload -ContentType "application/json"

    Log "Next instance deployment triggered for $userRepo (workflow: $WORKFLOW_FILE)"
}
catch {
    Fail "Deployment trigger failed: $_"
}

# === SHUTDOWN MONITOR ===
while ((Get-Date) -lt $shutdownTime) {
    $now       = Get-Date
    $elapsed   = [math]::Round(($now - $startTime).TotalMinutes, 1)
    $remaining = [math]::Round(($endTime - $now).TotalMinutes, 1)
    Log "Final phase status: $elapsed min elapsed | $remaining min until full shutdown"

    $waitMinutes = Get-Random -Minimum 15 -Maximum 30
    Start-Sleep -Seconds ($waitMinutes * 60)
}

# === TERMINATION ===
Log "Decommissioning EnigMano instance $INSTANCE_ID"
if ($RUNNER_ENV -eq "self-hosted") {
    Stop-Computer -Force
} else {
    Log "Termination skipped on hosted environment. Process exit."
    Exit
}
