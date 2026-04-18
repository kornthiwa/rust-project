# Runs every *.sql under each migrations/ folder in name order against the matching Compose Postgres service.
# auth-db / users-db each own one database (POSTGRES_DB); use apply-migrations after fresh volumes if needed.
$ErrorActionPreference = "Stop"
$root = Join-Path $PSScriptRoot ".."
Set-Location $root

function Ensure-Database {
    param([string]$ComposeService, [string]$Name)
    $check = "SELECT 1 FROM pg_database WHERE datname = '$Name'"
    $exists = (docker compose exec -T $ComposeService psql -U admin -d postgres -tAc $check).Trim()
    if ($exists -eq "1") { return }
    Write-Host "Creating database $Name on $ComposeService"
    docker compose exec -T $ComposeService psql -U admin -d postgres -v ON_ERROR_STOP=1 -c "CREATE DATABASE $Name OWNER admin;"
}

Ensure-Database "users-db" "users_service"
Ensure-Database "auth-db" "auth_service"

$migrationSets = @(
    @{ Database = "users_service"; ComposeService = "users-db"; RelativeDir = "users-service\migrations" }
    @{ Database = "auth_service"; ComposeService = "auth-db"; RelativeDir = "auth-service\migrations" }
)

foreach ($set in $migrationSets) {
    $dir = Join-Path $root $set.RelativeDir
    if (-not (Test-Path $dir)) {
        Write-Warning "Skip: folder not found: $dir"
        continue
    }

    $files = @(Get-ChildItem -Path $dir -Filter "*.sql" -File | Sort-Object Name)
    if ($files.Count -eq 0) {
        Write-Host "No *.sql in $($set.RelativeDir)"
        continue
    }

    foreach ($file in $files) {
        Write-Host "[$($set.Database)] $($file.Name)"
        Get-Content -Path $file.FullName -Raw | docker compose exec -T $set.ComposeService psql -U admin -d $set.Database -v ON_ERROR_STOP=1
    }
}

Write-Host "Done."
