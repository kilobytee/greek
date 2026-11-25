$ErrorActionPreference = 'Stop'

$envFile = Join-Path $PSScriptRoot '.env'
if (-not (Test-Path $envFile)) {
    throw "No se encontró $envFile"
}

$secure = Read-Host -AsSecureString -Prompt 'API key de OpenCode Zen'
$ptr = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
$key = [System.Runtime.InteropServices.Marshal]::PtrToStringBSTR($ptr)
[System.Runtime.InteropServices.Marshal]::ZeroFreeBSTR($ptr)

if ([string]::IsNullOrWhiteSpace($key)) {
    throw 'La key está vacía.'
}

$model = Read-Host -Prompt 'Modelo exacto de OpenCode Zen (Enter para conservar el actual)'

$lines = Get-Content $envFile | ForEach-Object {
    if ($_ -match '^OPENCODE_ZEN_API_KEY=') {
        "OPENCODE_ZEN_API_KEY=$key"
    } elseif (-not [string]::IsNullOrWhiteSpace($model) -and $_ -match '^OPENCODE_ZEN_MODEL=') {
        "OPENCODE_ZEN_MODEL=$($model.Trim())"
    } else {
        $_
    }
}

[System.IO.File]::WriteAllLines($envFile, $lines, [System.Text.UTF8Encoding]::new($false))
Write-Output 'Listo. Reinicia Greek para aplicar el cambio.'
