
cargo build;
cargo build --release;

$version = (git tag).Split("\n")[-1].Trim();

$debug = "$env:TEMP\win-gnome-debug.exe";
Copy-Item -Path .\target\debug\win-gnome.exe -Destination $debug
Compress-Archive -LiteralPath .\target\release\win-gnome.exe, .\install.ps1, .\uninstall.ps1, $debug  -DestinationPath .\target\win-gnome$version.zip -Force
Remove-Item -Path $debug
