param(
    [Parameter(Mandatory=$true)][string]$Zig,
    [Parameter(Mandatory=$true)][string]$LibraryDirectory,
    [string]$Prefix = (Join-Path $env:LOCALAPPDATA 'sm-decomp-tools/lakesnes-spc-1da16a2')
)
$ErrorActionPreference = 'Stop'
$revision = '1da16a22db4d0f2e7f6960a9846ef4e1f3acbd29'
$source = Join-Path $Prefix 'source/snes'
New-Item -ItemType Directory -Force -Path $source, $LibraryDirectory | Out-Null
$files = @{
    'spc.c' = 'fc9dfd730114d5c42c7cbde884091ff53e2e64b4237861f48d56b83d9ff123b7'
    'apu.h' = '1e63389e7e308fff401fe0c62b15623ac5aba4ca983669a67d728ab2192f4fc0'
    'cart.h' = '9723504d49ed9c6b9dc6e36068e0302c7428692676383159c3ceafaedf72a75e'
    'cpu.h' = '6ee238c3272ee781ccbf92ddecbe8d857548665ab15d06852f16048ab47ddea1'
    'dma.h' = 'b70680d974f0776d2aaa14fc5c96ebc55a9cae6a051838ad6db04e90c5766e5d'
    'dsp.h' = '9b4b0c6e554e5bf8dfde03be0da90f691270f9bfdeb9c32ca414a54345add6ba'
    'input.h' = 'b5f70fdd1817f4f2a62095a647b6b96b1d9cbb6b19d48cf68c3ada2871d9103b'
    'ppu.h' = 'bc7a46cafd27876fb196eced73479af796a3699df6469ac13985b7e4066d63d0'
    'snes.h' = 'd23d09ba03d058e35aa5ce4947855a39ad36b80fd8b1ef537c51c3a14f4ea81f'
    'spc.h' = 'fe42e827ea5447fc53ef7175b6bfd0471f4fa55e2bdbba15d857ad52629d4f2b'
}
foreach ($name in $files.Keys) {
    $file = Join-Path $source $name
    if (!(Test-Path -LiteralPath $file)) {
        Invoke-WebRequest "https://raw.githubusercontent.com/elzo-d/LakeSnes/$revision/snes/$name" -OutFile $file
    }
    if ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant() -ne $files[$name]) {
        throw "Upstream source checksum mismatch: $name"
    }
}
$license = Join-Path $Prefix 'LICENSE.txt'
if (!(Test-Path -LiteralPath $license)) {
    Invoke-WebRequest "https://raw.githubusercontent.com/elzo-d/LakeSnes/$revision/LICENSE.txt" -OutFile $license
}
if ((Get-FileHash -LiteralPath $license -Algorithm SHA256).Hash.ToLowerInvariant() -ne '1e88db7a44bec196f647c7b0e281517600797f4db5f8460bb02510c5219b257c') {
    throw 'Upstream license checksum mismatch'
}
$object = Join-Path $LibraryDirectory 'lakesnes_spc.o'
& $Zig cc -target x86_64-windows-gnu -O2 -std=c11 -c (Join-Path $source 'spc.c') -o $object
if ($LASTEXITCODE -ne 0) { throw 'SPC compilation failed' }
& $Zig ar rcs (Join-Path $LibraryDirectory 'liblakesnes_spc.a') $object
if ($LASTEXITCODE -ne 0) { throw 'SPC static-library creation failed' }
Write-Output "Installed pinned LakeSnes SPC library in $LibraryDirectory"
Write-Output 'Keep this directory in LIBRARY_PATH alongside the existing CPU/SDL libraries.'
