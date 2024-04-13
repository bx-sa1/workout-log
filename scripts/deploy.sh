#!/bin/bash
cd backend
cargo build --release

cd ..
scp backend/target/release/backend frontend/* backend/workouts.db baba@192.168.1.96:\~/workout-log
ssh baba@192.168.1.96 "mkdir -p ~/.config/systemd/user" && scp workout-log-backend.service workout-log-frontend.service baba@192.168.1.96:\~/.config/systemd/user
