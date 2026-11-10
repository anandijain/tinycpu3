param(
    [string]$WaveFile = 'build/test_test.test_fetch/test.fst',
    [string]$SurferExe = "$env:USERPROFILE/Downloads/surfer_win/surfer.exe",
    [switch]$PrepareOnly
)

$ErrorActionPreference = 'Stop'
$projectRoot = $PSScriptRoot
$metadataPath = Join-Path $projectRoot 'build/surfer.ron'
$statePath = Join-Path $projectRoot 'build/state.bincode'
$wavePath = if ([IO.Path]::IsPathRooted($WaveFile)) {
    [IO.Path]::GetFullPath($WaveFile)
} else {
    [IO.Path]::GetFullPath((Join-Path $projectRoot $WaveFile))
}

foreach ($requiredFile in @($metadataPath, $statePath, $wavePath, $SurferExe)) {
    if (-not (Test-Path -LiteralPath $requiredFile -PathType Leaf)) {
        throw "Missing file: $requiredFile"
    }
}

# Swim emits a RON map of quoted waveform paths to quoted Spade unit names.
# Keep the Linux entries so the same metadata still works inside WSL.
$metadata = Get-Content -LiteralPath $metadataPath -Raw
$topMap = [regex]::Match($metadata, '(?s)top_names\s*:\s*\{(?<entries>.*?)\}')
if (-not $topMap.Success) {
    throw "No top_names map in $metadataPath. Run swim test first."
}
$pairPattern = '"(?:\\.|[^"\\])*"\s*:\s*"(?:\\.|[^"\\])*"'
$entries = [ordered]@{}
foreach ($pair in [regex]::Matches($topMap.Groups['entries'].Value, $pairPattern)) {
    $parts = [regex]::Matches($pair.Value, '"(?:\\.|[^"\\])*"')
    $path = ConvertFrom-Json $parts[0].Value
    $top = ConvertFrom-Json $parts[1].Value
    $entries[$path] = $top
}
if ($entries.Count -eq 0) {
    throw "No waveform entries in $metadataPath. Run swim test first."
}

$wslRoot = [regex]::Match($projectRoot, '^\\\\(?:wsl\.localhost|wsl\$)\\(?<distro>[^\\]+)(?<path>\\.*)$')
if (-not $wslRoot.Success) {
    throw 'Run this launcher from the project through its \\wsl.localhost\<distro> path.'
}
$linuxRoot = $wslRoot.Groups['path'].Value.Replace('\', '/')
foreach ($path in @($entries.Keys)) {
    if ($path.StartsWith("$linuxRoot/", [StringComparison]::Ordinal)) {
        $relative = $path.Substring($linuxRoot.Length + 1).Replace('/', '\')
        $windowsPath = [IO.Path]::GetFullPath((Join-Path $projectRoot $relative))
        $entries[$windowsPath] = $entries[$path]
        $entries['\\?\UNC\' + $windowsPath.Substring(2)] = $entries[$path]
    }
}
if (-not $entries.Contains($wavePath)) {
    throw "No Spade top-level unit recorded for $wavePath. Run its swim test first."
}

$lines = @('(', '    state_file: "build/state.bincode",', '    top_names: {')
foreach ($entry in $entries.GetEnumerator()) {
    $key = ConvertTo-Json -InputObject ([string]$entry.Key) -Compress
    $value = ConvertTo-Json -InputObject ([string]$entry.Value) -Compress
    $lines += "        ${key}: ${value},"
}
$lines += @('    },', ')')
[IO.File]::WriteAllLines($metadataPath, $lines, [Text.UTF8Encoding]::new($false))

if (-not $PrepareOnly) {
    Push-Location -LiteralPath $projectRoot
    try {
        & $SurferExe $wavePath
    } finally {
        Pop-Location
    }
}
