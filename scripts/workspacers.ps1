# Override the path to the exe(if you don't want to add it to your path)
#
$WsExe = "workspacers-cli"

# Override the json file used. If blank, $XDG_DATA_HOME will be used:
# | Platform | Example                                       |
# |----------|-----------------------------------------------|
# | Linux    | /home/alice/.local/share/workspacers.json     |
# | Windows  | C:\Users\Alice\AppData\Local\workspacers.json |
#
$WsJson = ""

# Set how to open files
#
$FileOpen = "nvim {0}"
# $FileOpen = "code {0}"
# $FileOpen = "notepad {0}"
# $FileOpen = "xdg-open {0}"

# Set how to open directories
#
$DirOpen = "cd {0}; ls"
# $DirOpen = "explorer {0}"
# $DirOpen = "xdg-open {0}"

# Code
function ws {
    if (-not [string]::IsNullOrWhiteSpace($WsJson)) { # Add --json-file arg
        $wsArgs += @("--json-file=$WsJson")
    }

    if ($args[0] -eq "-a") { # Add Command
        $wsArgs += @($args)
        & $WsExe $wsArgs # `Add` needs to call directly without capturing output
        return
    } 
    if ($args[0] -eq "-j") { # Print Json File
        $wsArgs += @($args)
        Invoke-Expression ($FileOpen -f (& $WsExe $wsArgs))
        return
    } 

    # Call App and capture output
    $path = & $WsExe $wsArgs
    if ($LASTEXITCODE -eq 0 -and $path) {
        if (Test-Path -Path $path) { # Open Path output
            if (Test-Path -Path $path -PathType Leaf) {
                Invoke-Expression ($FileOpen -f $path.Trim())
            } else {
                Invoke-Expression ($DirOpen -f $path.Trim())
            }
        } else {
            Write-Host "Path does not exist: $path"
        }
    }
}
