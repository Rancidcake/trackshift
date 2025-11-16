# Script to fix base64ct dependency issue
# This patches the base64ct Cargo.toml to use edition 2021 instead of 2024

$base64ctPath = "$env:USERPROFILE\.cargo\registry\src\index.crates.io-*\base64ct-1.8.0\Cargo.toml"
$files = Get-Item $base64ctPath -ErrorAction SilentlyContinue

if ($files) {
    foreach ($file in $files) {
        Write-Host "Fixing $($file.FullName)"
        $content = Get-Content $file.FullName -Raw
        $content = $content -replace 'edition = "2024"', 'edition = "2021"'
        Set-Content -Path $file.FullName -Value $content -NoNewline
        Write-Host "Fixed!"
    }
} else {
    Write-Host "base64ct-1.8.0 not found in registry. It will be downloaded and fixed on first build."
}

