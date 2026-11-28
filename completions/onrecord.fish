# fish completion for onrecord
complete -c onrecord -n '__fish_use_subcommand' -a 'add' -d 'onrecord add'
complete -c onrecord -n '__fish_use_subcommand' -a 'get' -d 'onrecord get'
complete -c onrecord -n '__fish_use_subcommand' -a 'subject' -d 'onrecord subject'
complete -c onrecord -n '__fish_use_subcommand' -a 'due' -d 'onrecord due'
complete -c onrecord -n '__fish_use_subcommand' -a 'check' -d 'onrecord check'
complete -c onrecord -s h -l help -d 'Show help'
complete -c onrecord -n '__fish_seen_subcommand_from add' -l 'text' -r
complete -c onrecord -n '__fish_seen_subcommand_from add' -l 'asserter' -r
complete -c onrecord -n '__fish_seen_subcommand_from add' -l 'at' -r
complete -c onrecord -n '__fish_seen_subcommand_from add' -l 'evidence' -r
complete -c onrecord -n '__fish_seen_subcommand_from add' -l 'supersedes' -r
complete -c onrecord -n '__fish_seen_subcommand_from add' -l 'subject' -r
complete -c onrecord -n '__fish_seen_subcommand_from add' -l 'due' -r
