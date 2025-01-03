## Tmux

You can use **navi** as a [Tmux](https://github.com/tmux/tmux/wiki) widget to reach your Vim commands, often used SQL queries, etc. in any command-line app even in SSH sessions.

Add these lines to your Tmux config file to access **navi** by pressing `prefix + C-g`.

```sh
bind-key -N "Open Navi (cheat sheets)" -T prefix C-g split-window \
  "$SHELL --login -i -c 'navi --print | head -n 1 | tmux load-buffer -b tmp - ; tmux paste-buffer -p -t {last} -b tmp -d'"
```

Example cheatsheet:

```sh
% vim 

# Quit without save
qa!

# Delete a paragraph
normal dap

# Generate sequence of numbers
put =range(<start>, <stop>)

% postgresql

# Describe table columns in `psql` or `pgcli`
select 
   table_name, 
   column_name, 
   data_type 
from 
   information_schema.columns
where 
   table_name = '<table>';
```
