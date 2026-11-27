# PowerShell completion for onrecord
Register-ArgumentCompleter -Native -CommandName onrecord -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    $candidates = @('add', 'get', 'subject', 'due', 'check', '--text', '--asserter', '--at', '--evidence', '--supersedes', '--subject', '--due', '-h', '--help')
    $candidates | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
    }
}
