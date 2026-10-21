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
    'snes/ppu.c' = '29062d3711eff663a4411a39ebc02c8eeed529ba2363b3c81a677caed9277b6c'
    'snes/ppu.h' = 'c380d08458349a820a71418ac485355555532757ae74e4fc70e9cb8b08ba8da5'
    'snes/snes.h' = '945e7abd47bebe8e89c2dbd2a19acd50d68da762e0166f86237ad21caa69822b'
    'snes/dsp.c' = 'ac25349b4e5758dd79529e9601442fb424a879f25a47cec15203679486cbeb1c'
    'snes/dsp.h' = '15fb73eeff7a0c7dcb193e4c0715be389bb49c8e6283dd1a82f49d40ead0caad'
    'snes/apu.h' = '6cfdb3f5a9086b707bd92af3f8f8d1a0014152eefc98d64ebf59403070464b89'
    'LICENSE.txt' = '9da2262b7b91dee7309173702f28192c5851197efac56e3ce3b5b751abf83bdc'
}
foreach ($name in $files.Keys) {
    $file = Join-Path $source $name
    if (!(Test-Path -LiteralPath $file)) {
        Invoke-WebRequest "https://raw.githubusercontent.com/angelo-wf/LakeSnes/$revision/$name" -OutFile $file
    }
    if ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant() -ne $files[$name]) {
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
