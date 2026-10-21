param(
    [Parameter(Mandatory=$true)][string]$Zig,
    [string]$Prefix = (Join-Path $env:LOCALAPPDATA 'sm-decomp-tools/lakesnes-cpu-048a0d7'),
    [string]$LibraryDirectory
)
$ErrorActionPreference = 'Stop'
$revision = '048a0d72568668f74e0cdf371dcd9e8efd614965'
$source = Join-Path $Prefix 'source/snes'
$library = if ($LibraryDirectory) { $LibraryDirectory } else { Join-Path $Prefix 'lib' }
New-Item -ItemType Directory -Force -Path $source, $library | Out-Null
$files = @{
    'cpu.c' = '45200e56488392a25f2664caedb4edb7782ac78197a28972ccf5612b0088a470'
    'cpu.h' = '4dcce50ce511cf44e115c9384bcb5cf0e057d2158581b69f490c12a613941b19'
    'statehandler.c' = '2bf94ba80e0c7ca8363ab2a1f7d3e7ecb076278c9339990a73141641b772f1c0'
    'statehandler.h' = '6e1b3affd60dd242bfa2dd5b25c67edfb8663d28f6c2b89e8db1a15b7b7d7249'
}
foreach ($name in $files.Keys) {
    $file = Join-Path $source $name
    if (!(Test-Path -LiteralPath $file)) {
        Invoke-WebRequest "https://raw.githubusercontent.com/dinkc64/LakeSnes/$revision/snes/$name" -OutFile $file
    }
    if ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant() -ne $files[$name]) {
        throw "Upstream source checksum mismatch: $name"
    }
}
$license = Join-Path $Prefix 'LICENSE.txt'
if (!(Test-Path -LiteralPath $license)) {
    Invoke-WebRequest "https://raw.githubusercontent.com/dinkc64/LakeSnes/$revision/LICENSE.txt" -OutFile $license
}
if ((Get-FileHash -LiteralPath $license -Algorithm SHA256).Hash.ToLowerInvariant() -ne '27568d73a4ce17ced1bf37e86bfee6fcb9b0f6a27de517ddbc16c946c5c509fd') {
    throw 'Upstream license checksum mismatch'
}
foreach ($name in @('cpu', 'statehandler')) {
    & $Zig cc -target x86_64-windows-gnu -O2 -std=c11 -c (Join-Path $source "$name.c") -o (Join-Path $library "$name.o")
    if ($LASTEXITCODE -ne 0) { throw "C compilation failed: $name" }
}
& $Zig ar rcs (Join-Path $library 'liblakesnes_cpu.a') (Join-Path $library 'cpu.o') (Join-Path $library 'statehandler.o')
if ($LASTEXITCODE -ne 0) { throw 'CPU static-library creation failed' }
Write-Output "Installed pinned LakeSnes CPU library in $library"
Write-Output 'Use -LibraryDirectory with the configured SDL native-library directory for normal frontend builds.'
Write-Output 'Add the library directory to LIBRARY_PATH for the normal Cargo linker.'
