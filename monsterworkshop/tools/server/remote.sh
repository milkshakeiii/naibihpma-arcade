#!/bin/bash
# Run on the server as root by deploy.ps1: builds the uploaded source as the mw user, then
# restarts the game server in a tmux session named mw, letting the old one save first.
# `sudo -u mw tmux attach -t mw` shows its console; typing STOP there saves and stops it.
set -euo pipefail

# A new server is still setting itself up for a minute or two.
cloud-init status --wait >/dev/null 2>&1 || true
if ! command -v cc >/dev/null || ! command -v tmux >/dev/null; then
    apt-get -o DPkg::Lock::Timeout=600 update -q
    DEBIAN_FRONTEND=noninteractive apt-get -o DPkg::Lock::Timeout=600 install -y -q build-essential tmux
fi
id mw >/dev/null 2>&1 || useradd --create-home --shell /bin/bash mw
as_mw() { sudo -u mw -H bash -c "$1"; }
as_mw 'test -x ~/.cargo/bin/cargo || curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y -q --profile minimal'

rm -rf /home/mw/src
mkdir /home/mw/src
tar -xzf /tmp/mw.tar.gz -C /home/mw/src
chown -R mw:mw /home/mw/src
# The build lives outside the source, so a new upload doesn't build everything again.
as_mw 'cd ~/src/monsterworkshop && CARGO_TARGET_DIR=~/target ~/.cargo/bin/cargo build --release -q'

if as_mw 'tmux has-session -t mw' 2>/dev/null; then
    as_mw 'tmux send-keys -t mw STOP Enter'
    for _ in $(seq 60); do as_mw 'tmux has-session -t mw' 2>/dev/null || break; sleep 1; done
fi
as_mw 'tmux new-session -d -s mw "~/target/release/monster_workshop --server 2>&1 | tee -a ~/server.log"'
sleep 2
as_mw 'tail -n 3 ~/server.log'
