# Runs every *.sql under each *-service/migrations folder in name order
# against the matching Compose Postgres service (<name>-db) and database (<name>_service).
# New services are discovered automatically when they add a migrations folder.
$ErrorActionPreference = "Stop"
$root = Join-Path $PSScriptRoot ".."
Set-Location $root

function Get-ComposeServices {
    $services = docker compose config --services 2>$null
    if (-not $services) {
        throw "Unable to read docker compose services. Run from repo root and ensure docker compose is available."
    }
    return @($services | ForEach-Object { $_.Trim() } | Where-Object { $_ })
}

function Ensure-Database {
    param([string]$ComposeService, [string]$Name)
    $check = "SELECT 1 FROM pg_database WHERE datname = '$Name'"
    $exists = (docker compose exec -T $ComposeService psql -U admin -d postgres -tAc $check).Trim()
    if ($exists -eq "1") { return }
    Write-Host "Creating database $Name on $ComposeService"
    docker compose exec -T $ComposeService psql -U admin -d postgres -v ON_ERROR_STOP=1 -c "CREATE DATABASE $Name OWNER admin;"
}

function Get-MigrationSets {
    $serviceDirs = @(Get-ChildItem -Path $root -Directory | Where-Object {
        $_.Name -like "*-service" -and (Test-Path (Join-Path $_.FullName "migrations"))
    })

    $sets = @()
    foreach ($dir in $serviceDirs) {
        $serviceName = $dir.Name
        $prefix = $serviceName -replace "-service$", ""
        if (-not $prefix) { continue }

        $sets += @{
            Database = "${prefix}_service"
            ComposeService = "${prefix}-db"
            RelativeDir = "$serviceName\migrations"
        }
    }

    return @($sets | Sort-Object RelativeDir)
}

$composeServices = Get-ComposeServices
$migrationSets = Get-MigrationSets

foreach ($set in $migrationSets) {
    if ($composeServices -notcontains $set.ComposeService) {
        Write-Warning "Skip: compose service not found: $($set.ComposeService) for $($set.RelativeDir)"
        continue
    }

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
