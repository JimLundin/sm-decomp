param(
    [Parameter(Mandatory=$true)][string]$Zig,
    [string]$Prefix = (Join-Path $env:LOCALAPPDATA 'sm-decomp-tools/lakesnes-backend-28b8fbc'),
    [string]$LibraryDirectory
)
$ErrorActionPreference = 'Stop'
$revision = '28b8fbc15ad15afff003165ff4f2a8adb1c93d18'
$source = Join-Path $Prefix 'source'
$library = if ($LibraryDirectory) { $LibraryDirectory } else { Join-Path $Prefix 'lib' }
New-Item -ItemType Directory -Force -Path (Join-Path $source 'snes'), $library | Out-Null
$files = @{
    'snes/ppu.c' = '1713d72737ce85996da96359fe141f472845a5b374408c30b866a724cb25824a'
    'snes/ppu.h' = 'bc7a46cafd27876fb196eced73479af796a3699df6469ac13985b7e4066d63d0'
    'snes/snes.h' = '47f15f42667bb749263022244575b053f116bc75f4d45b9fd32d46e00466795e'
    'snes/dsp.c' = 'a2710134d7b648ce26e720af8ff56347cab537ddc4ce121894b761624db2fb0b'
    'snes/dsp.h' = '9b4b0c6e554e5bf8dfde03be0da90f691270f9bfdeb9c32ca414a54345add6ba'
    'snes/apu.h' = '1e63389e7e308fff401fe0c62b15623ac5aba4ca983669a67d728ab2192f4fc0'
    'LICENSE.txt' = '1e88db7a44bec196f647c7b0e281517600797f4db5f8460bb02510c5219b257c'
}
foreach ($name in $files.Keys) {
    $file = Join-Path $source $name
    if (!(Test-Path -LiteralPath $file)) {
        Invoke-WebRequest "https://raw.githubusercontent.com/angelo-wf/LakeSnes/$revision/$name" -OutFile $file
    }
    # Upstream downloads use LF; the established local source checkout used CRLF.
    # Those files were verified identical except for line endings. Hash canonical
    # LF bytes while leaving the compiler input and cached files untouched.
    $byteEncoding = [System.Text.Encoding]::GetEncoding(28591)
    $sourceText = $byteEncoding.GetString([System.IO.File]::ReadAllBytes($file)).Replace("`r`n", "`n")
    $sourceBytes = $byteEncoding.GetBytes($sourceText)
    $sourceSha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $sourceHash = [BitConverter]::ToString($sourceSha.ComputeHash($sourceBytes)).Replace('-', '').ToLowerInvariant()
    } finally {
        $sourceSha.Dispose()
    }
    if ($sourceHash -ne $files[$name]) {
        throw "Upstream source checksum mismatch: $name"
    }
}
$bridge = Join-Path $PSScriptRoot '../sm/src/snes/backend.c'
$object = Join-Path $library 'backend.o'
$stamp = Join-Path $library 'backend_source.h'
$bridgeHash = (Get-FileHash -LiteralPath $bridge -Algorithm SHA256).Hash.ToLowerInvariant()
Set-Content -LiteralPath $stamp -Value ('#define SM_BACKEND_SOURCE_HASH "' + $bridgeHash + '"') -Encoding ascii
& $Zig cc -target x86_64-windows-gnu -O2 -std=c11 -fexceptions -ffunction-sections -fdata-sections -include $stamp -I (Join-Path $source 'snes') -c $bridge -o $object
if ($LASTEXITCODE -ne 0) { throw 'Backend C compilation failed' }
& $Zig ar rcs (Join-Path $library 'liblakesnes_backend.a') $object
if ($LASTEXITCODE -ne 0) { throw 'Backend static-library creation failed' }
Write-Output "Installed pinned LakeSnes backend library in $library"
Write-Output 'Add the library directory to LIBRARY_PATH for the normal Cargo linker.'
