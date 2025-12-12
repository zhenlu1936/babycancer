#!/bin/bash

# BabyCancer REPL Demo - Command Reminder
# Shows the commands to enter in the REPL

cat << 'EOF'
=== BabyCancer REPL Demo Commands ===

1. Configure basic backup:
   config --source-path demo_repl/source --dest-path demo_repl/backup

2. Run backup:
   backup

3. Configure compressed backup:
   config --source-path demo_repl/source --dest-path demo_repl/backup --tar true --gzip true

4. Run compressed backup:
   backup

5. Exit:
   exit

EOF
