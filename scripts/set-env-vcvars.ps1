# .env の VCVARS64_BAT を書き換え（重複しないよう1行だけに）
# 用法: .\set-env-vcvars.ps1 "E:\path\to\vcvars64.bat"
param([Parameter(Mandatory = $true)][string]$BatchPath)

$projectRoot = Split-Path $PSScriptRoot -Parent
$envPath = Join-Path $projectRoot '.env'
$examplePath = Join-Path $projectRoot '.env.example'

if (!(Test-Path $envPath)) {
    Copy-Item $examplePath $envPath
}

$content = Get-Content $envPath -Raw
$content = $content -replace '(?m)^\s*VCVARS64_BAT=.*\r?\n?', ''
Set-Content $envPath $content
Add-Content $envPath "VCVARS64_BAT=$BatchPath"
