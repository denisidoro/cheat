function _navi_smart_replace
    set --local query (commandline --current-process | string trim)

    if test -n "$query"
        set --local best_match (navi --print --query "$query" --best-match)
        if test -n "$best_match"
            commandline --current-process $best_match
            return
        end
    end

    set --local candidate (navi --print --query "$query")
    if test -n "$candidate"
        commandline --current-process $candidate
    end
end

bind \cg _navi_smart_replace
bind --mode insert \cg _navi_smart_replace
