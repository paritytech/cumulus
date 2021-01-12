#!/bin/bash

# script that setups a tmux session with three panes that attach to the log files 
# of alice (relay chain) and the two parachains (id=1862, id=1863)

if tmux has-session -t polkadot_logger ; then
  echo "detected existing polkadot logger session, attaching..."
else
  # or start it up freshly
  tmux new-session -d -s polkadot_logger \; \
    split-window -v \; \
    split-window -v \; \
    select-layout even-vertical \; \
    send-keys -t polkadot_logger:0.0 'tail -f ./alice.log' C-m \; \
    send-keys -t polkadot_logger:0.1 'tail -f ./1862.log' C-m \; \
    send-keys -t polkadot_logger:0.2 'tail -f ./1863.log' C-m

    # Attention: Depending on your tmux conf, indexes may start at 1

    tmux setw -g mouse on
fi
tmux attach-session -d -t polkadot_logger
