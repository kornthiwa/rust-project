# Applies SQL migrations under each *-service/migrations folder.
#
# sqlx (Rust): services call sqlx::migrate!("./migrations") on startup; migration files should stay
# idempotent (e.g. CREATE TABLE IF NOT EXISTS) where re-runs are possible.
#
# This script offers two modes:
# 1) Preferred — `sqlx migrate run` from each service crate (updates _sqlx_migrations; matches sqlx-cli).
#    Install:  cargo install sqlx-cli --no-default-features --features rustls,postgres
#    Requires Postgres reachable on localhost with the ports from docker-compose.yml.
# 2) Fallback — pipe each *.sql into `docker compose exec ... psql` (no sqlx-cli; does NOT write
#    _sqlx_migrations; fine for raw DDL when apps will still run migrate! on boot).
#
# Compose DB services: auth-db, users-db, messages-db → databases auth_service, users_service, messages_service.
param(
    [switch]$ForceDockerPsql
)

$ErrorActionPreference = "Stop"
$root = Join-Path $PSScriptRoot ".."
Set-Location $root

# Optional: only services listed here use `sqlx migrate run` from the host (needs matching compose port).
# Any other *-service/migrations is still picked up automatically; those run via docker psql fallback.
# Host URLs for local docker-compose Postgres (see repo docker-compose.yml port mappings).
$Script:DefaultDatabaseUrls = @{
    "auth-service"     = "postgresql://admin:admin1234@127.0.0.1:5432/auth_service"
    "users-service"    = "postgresql://admin:admin1234@127.0.0.1:5433/users_service"
    "messages-service" = "postgresql://admin:admin1234@127.0.0.1:5435/messages_service"
}

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
            ServiceName    = $serviceName
            Database       = "${prefix}_service"
            ComposeService = "${prefix}-db"
            MigrationsDir    = (Join-Path $dir.FullName "migrations")
        }
    }

    return @($sets | Sort-Object ServiceName)
}

function Invoke-SqlxMigrateFromCrate {
    param(
        [string]$ServiceName,
        [string]$CrateRoot,
        [string]$DatabaseUrl
    )
    Write-Host "[sqlx] $ServiceName → migrate run ($DatabaseUrl)"
    Push-Location $CrateRoot
    try {
        $prev = $env:DATABASE_URL
        $env:DATABASE_URL = $DatabaseUrl
        & sqlx migrate run
        if ($LASTEXITCODE -ne 0) { throw "sqlx migrate run failed for $ServiceName (exit $LASTEXITCODE)" }
    }
    finally {
        if ($null -eq $prev) { Remove-Item Env:DATABASE_URL -ErrorAction SilentlyContinue }
        else { $env:DATABASE_URL = $prev }
        Pop-Location
    }
}

function Invoke-DockerPsqlFiles {
    param(
        [string]$Database,
        [string]$ComposeService,
        [string]$MigrationsDir
    )

    $files = @(Get-ChildItem -Path $MigrationsDir -Filter "*.sql" -File | Sort-Object Name)
    if ($files.Count -eq 0) {
        Write-Host "No *.sql in $MigrationsDir"
        return
    }

    foreach ($file in $files) {
        Write-Host "[$Database] psql ← $($file.Name)"
        Get-Content -Path $file.FullName -Raw | docker compose exec -T $ComposeService psql -U admin -d $Database -v ON_ERROR_STOP=1
        if ($LASTEXITCODE -ne 0) { throw "psql failed on $($file.Name) for $Database (exit $LASTEXITCODE)" }
    }
}

$composeServices = Get-ComposeServices
$migrationSets = Get-MigrationSets

$sqlxAvailable = -not $ForceDockerPsql -and ($null -ne (Get-Command sqlx -ErrorAction SilentlyContinue))
if ($ForceDockerPsql) {
    Write-Host "Using docker psql only (-ForceDockerPsql)."
}
elseif (-not $sqlxAvailable) {
    Write-Host "sqlx CLI not on PATH; using docker psql. Install sqlx-cli for migrate run + _sqlx_migrations sync."
}

foreach ($set in $migrationSets) {
    if ($composeServices -notcontains $set.ComposeService) {
        Write-Warning "Skip: compose service not found: $($set.ComposeService) for $($set.ServiceName)/migrations"
        continue
    }

    if (-not (Test-Path $set.MigrationsDir)) {
        Write-Warning "Skip: folder not found: $($set.MigrationsDir)"
        continue
    }

    Ensure-Database -ComposeService $set.ComposeService -Name $set.Database

    $crateRoot = Join-Path $root $set.ServiceName
    $useSqlxThis = $sqlxAvailable -and $Script:DefaultDatabaseUrls.ContainsKey($set.ServiceName)

    if ($useSqlxThis) {
        Invoke-SqlxMigrateFromCrate -ServiceName $set.ServiceName -CrateRoot $crateRoot -DatabaseUrl $Script:DefaultDatabaseUrls[$set.ServiceName]
    }
    else {
        Invoke-DockerPsqlFiles -Database $set.Database -ComposeService $set.ComposeService -MigrationsDir $set.MigrationsDir
    }
}

Write-Host "Done."
