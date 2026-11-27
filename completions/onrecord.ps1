# PowerShell completion for onrecord
Register-ArgumentCompleter -Native -CommandName onrecord -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    $elements = $commandAst.CommandElements
    $subMap = @{ 'add' = @('--text', '--asserter', '--at', '--evidence', '--supersedes', '--subject', '--due', '-h', '--help'), 'get' = @('-h', '--help'), 'subject' = @('-h', '--help'), 'due' = @('-h', '--help'), 'check' = @('-h', '--help') }
    if ($elements.Count -le 1 -or ($elements.Count -eq 2 -and $wordToComplete.Length -gt 0 -and -not $subMap.ContainsKey($elements[1].Extent.Text))) {
        $candidates = @('add', 'get', 'subject', 'due', 'check', '-h', '--help')
    } else {
        $sub = $elements[1].Extent.Text
        if ($subMap.ContainsKey($sub)) {
            $candidates = $subMap[$sub]
        } else {
            $candidates = @('-h', '--help')
        }
    }
    $candidates | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
    }
}
