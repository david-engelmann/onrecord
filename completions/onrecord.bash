# bash completion for onrecord
_onrecord() {
    local cur prev words cword
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "add get subject due check -h --help" -- "${cur}") )
        return 0
    fi
    case "${cur}" in
        -*)
            COMPREPLY=( $(compgen -W "--text --asserter --at --evidence --supersedes --subject --due -h --help" -- "${cur}") )
            return 0
            ;;
    esac
    COMPREPLY=( $(compgen -f -- "${cur}") )
}
complete -F _onrecord onrecord
