#Requires -RunAsAdministrator
$targetname = "win-gnome"
$target = "$targetname.exe"
$targetfinal = ".\target\release\$target"
$destination = "$Env:Programfiles\WinGnome"
$destfinal = "$destination\$target"
$taskname = "WinGnome"

Get-Process $targetname -OutVariable process_exists >$null
if($process_exists){
    Write-Host "Killing any currently running instance of win-gnome.exe..."
    Stop-Process -Name $targetname -Force
}
Write-Host "Creating $destination"
New-Item -ItemType Directory -Force -Path $destination >$null
Write-Host "Placing $target in $destination"
Copy-Item -Path $targetfinal -Destination $destination -Force
$trigger = New-ScheduledTaskTrigger -AtLogOn # Specify the trigger settings
$user = "$env:USERDOMAIN\$env:UserName" # Specify the account to run the script
$action = New-ScheduledTaskAction -Execute $destfinal # Specify what program to run and with its parameters
$description = "Start WinGnome at login"

Get-ScheduledTask -TaskName $taskname -ErrorAction SilentlyContinue -OutVariable task_exists > $null
if ($task_exists) {
    Write-Host "Removing previous  '$taskname' scheduled task"
    Unregister-ScheduledTask -TaskName $taskname -Confirm:$false
}

Write-Host "Registering new '$taskname' scheduled task"
Register-ScheduledTask -TaskName $taskname -Trigger $trigger -User $User -Action $action -Description $description -RunLevel Highest -Force >$null
Write-Host "Starting '$taskname'"
Start-ScheduledTask -TaskName $taskname