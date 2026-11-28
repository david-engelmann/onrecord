# bash completion for onrecord
_onrecord() {
    local cur prev sub sub_opts line
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "add get subject due check -h --help" -- "${cur}") )
        return 0
    fi
    sub="${COMP_WORDS[1]}"
    sub_opts="-h --help"
    case "${sub}" in
        add) sub_opts="--text --asserter --at --evidence --supersedes --subject --due -h --help" ;;
        get) sub_opts="-h --help" ;;
        subject) sub_opts="-h --help" ;;
        due) sub_opts="-h --help" ;;
        check) sub_opts="-h --help" ;;
    esac
    case "${cur}" in
        -*)
            COMPREPLY=( $(compgen -W "${sub_opts}" -- "${cur}") )
            return 0
            ;;
    esac
    compopt -o filenames 2>/dev/null || true
    while IFS= read -r line; do
        COMPREPLY+=( "${line}" )
    done < <(compgen -f -- "${cur}")
}
complete -F _onrecord onrecord
