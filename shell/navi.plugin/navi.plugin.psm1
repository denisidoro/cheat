
[CmdletBinding()]
param (
    ### Should we debug this PS Script ?
    ### Caution: Set-PSDebug is not limited to this script but enables session-wide calls
    ### Be sure to disable Verbose output before calling any other shell modules or scripts.
    [Parameter()]
    [bool]
    $VerboseOutput
)

if ($VerboseOutput) {
    ### Outputs the running code if required
    ###
    ### For more details on how it works, please see:
    ### - https://stackoverflow.com/a/41334715/13025136
    ###   An answer and explaination from @michael-sorens (https://stackoverflow.com/users/115690/michael-sorens)
    ###   on how Set-PSDebug relates to set-x in LINUX/UNIX environments.
    ###
    ### - https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/set-psdebug?view=powershell-7.4
    ###   Microsoft's Reference and documentation for the `Set-PSDebug` function.
    Set-PSDebug -Trace 1
} else {
    Set-PSDebug -Trace 0
}


### Initial code from @lurebat (https://github.com/lurebat/)
### See #570 (https://github.com/denisidoro/navi/issues/570) for its original contribution
function Invoke-NaviWidget {
    $ast = $tokens = $errors = $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref] $ast, [ref] $tokens, [ref] $errors, [ref] $cursor)

    $line = $ast.ToString().Trim()
    $output = $null

    if ([String]::IsNullOrEmpty($line)) {
        $output = navi --print
    }
    else {
        $best_match = (navi --print --best-match --query $line | Out-String).Trim()
        if ([String]::IsNullOrEmpty($best_match)) {
            $output = (navi --print --query "$line" | Out-String).Trim()
        }
        else {
            $output = $best_match
        }
    }

    [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()

    ### Handling the case when the user escapes without selecting any entry
    if (-Not([String]::IsNullOrEmpty($output))) {
        [Microsoft.PowerShell.PSConsoleReadLine]::Insert([String]$output)
    }
}