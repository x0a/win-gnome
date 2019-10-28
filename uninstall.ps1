# The following is to request admin priviledges if not currently admin
# Adapted from https://blogs.msdn.microsoft.com/virtual_pc_guy/2010/09/23/a-self-elevating-powershell-script/

$currentuser = [System.Security.Principal.WindowsIdentity]::GetCurrent()
$currentprincipal = new-object System.Security.Principal.WindowsPrincipal($currentuser)
$admin = [System.Security.Principal.WindowsBuiltInRole]::Administrator

if(!$currentprincipal.IsInRole($admin)){
    $nextprocess = new-object System.Diagnostics.ProcessStartInfo "PowerShell";
   
    # Specify the current script path and name as a parameter
    $nextprocess.Arguments = $myInvocation.MyCommand.Definition;
    
    # Indicate that the process should be elevated
    $nextprocess.Verb = "runas";
    
    # Start the new process
    [System.Diagnostics.Process]::Start($nextprocess);
    
    # Exit from the current, unelevated, process
    exit
}
# Uninstallation start

$targetname = "win-gnome"
$target = "$targetname.exe"
$destination = "$Env:Programfiles\WinGnome"
$taskname = "WinGnome"

Get-Process $targetname -Erroraction "silentlycontinue" -OutVariable process_exists >$null
if($process_exists){
    Write-Host "Killing any currently running instance of win-gnome.exe..."
    taskkill /im win-gnome.exe
}

Get-ScheduledTask -TaskName $taskname -ErrorAction SilentlyContinue -OutVariable task_exists > $null
if ($task_exists) {
    Write-Host "Removing previous  '$taskname' scheduled task"
    Unregister-ScheduledTask -TaskName $taskname -Confirm:$false
}
Write-Host "Removing installed directories"
Remove-Item -Recurse -Force $destination

Write-Host "Press any key to exit..."
[void][System.Console]::ReadKey($true)