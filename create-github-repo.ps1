# Create auxmos-bluemoon repo on GitHub (IO979996) and push
# Usage: .\create-github-repo.ps1
# Or with token: $env:GITHUB_TOKEN = "ghp_..."; .\create-github-repo.ps1

$repoName = "auxmos-bluemoon"
$token = $env:GITHUB_TOKEN

if (-not $token) {
    Write-Host "GITHUB_TOKEN not set. Opening GitHub 'New repository' page..."
    Start-Process "https://github.com/new?name=$repoName&description=Rust+atmospherics+fork+for+BlueMoon-Station"
    Write-Host "Create the repository (leave it empty), then run:"
    Write-Host '  git push -u bluemoon master'
    exit 1
}

$body = @{
    name        = $repoName
    description = "Rust atmospherics fork for BlueMoon-Station"
    private     = $false
} | ConvertTo-Json

$headers = @{
    "Authorization" = "token $token"
    "Accept"         = "application/vnd.github.v3+json"
}

try {
    $resp = Invoke-RestMethod -Uri "https://api.github.com/user/repos" -Method Post -Headers $headers -Body $body -ContentType "application/json"
    Write-Host "Created repository: $($resp.html_url)"
} catch {
    Write-Host "API error: $_"
    exit 1
}

Push-Location $PSScriptRoot
try {
    & git push -u bluemoon master
    if ($LASTEXITCODE -eq 0) { Write-Host "Done. Repo: https://github.com/IO979996/$repoName" }
} finally { Pop-Location }
