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
# Installation start
$targetname = "win-gnome"
$target = "$targetname.exe"
if(Test-Path ($PSScriptRoot + ".\target\release\$target")){
    $targetfinal = $PSScriptRoot + ".\target\release\$target"
}else{
    $targetfinal = $PSScriptRoot + ".\$target"
}
$destination = "$Env:Programfiles\WinGnome"
$destfinal = "$destination\$target"
$taskname = "WinGnome"

Get-Process $targetname -Erroraction "silentlycontinue" -OutVariable process_exists >$null
if($process_exists){
    Write-Host "Killing any currently running instance of win-gnome.exe..."
    taskkill /im win-gnome.exe
}
Write-Host "Creating $destination"
New-Item -ItemType Directory -Force -Path $destination >$null
Write-Host "Placing $target in $destination"
Copy-Item -Path $targetfinal -Destination $destination -Force
$trigger = New-ScheduledTaskTrigger -AtLogOn # Specify the trigger settings
$user = "$env:USERDOMAIN\$env:UserName" # Specify the account to run the script
$action = New-ScheduledTaskAction -Execute $destfinal # Specify what program to run and with its parameters
$settings = New-ScheduledTaskSettingsSet -ExecutionTimeLimit 0
$description = "Start WinGnome at login"

Get-ScheduledTask -TaskName $taskname -ErrorAction SilentlyContinue -OutVariable task_exists > $null
if ($task_exists) {
    Write-Host "Removing previous  '$taskname' scheduled task"
    Unregister-ScheduledTask -TaskName $taskname -Confirm:$false
}

Write-Host "Registering new '$taskname' scheduled task"
Register-ScheduledTask -TaskName $taskname -Trigger $trigger -User $User -Action $action -Description $description -RunLevel Highest -Settings $settings -Force >$null
Write-Host "Starting '$taskname'"
Start-ScheduledTask -TaskName $taskname

Write-Host "Press any key to exit..."
[void][System.Console]::ReadKey($true)