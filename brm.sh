#!/bin/bash

set -e

start() {
    echo "BRUME STARTING..."
    cd ~/brume && docker compose up -d
}

stop() {
    echo "BRUME STOPPING..."
    remove_jobs > /dev/null 2>&1
    cd ~/brume && docker compose down
}

restart() {
    stop
    start
}

remove_jobs() {
    # remove all jobs launched by brume
    # if we have contaiern we delete
    docker ps -f label=brume.dev/managed=true -q | ifne xargs docker rm -f
}

show_jobs() {
    # show all jobs launched by brume
    echo "BRUME JOBS"
    docker ps -f label=brume.dev/managed=true
}

nuke() {
    # remove all containers launched by brume
    echo "BRUME NUKING EVERYTHING..."
    stop > /dev/null 2>&1
    remove_jobs > /dev/null 2>&1
    cd ~/brume && docker compose down -v
}

orchestrator_logs() {
    docker logs brume-orchestrator -f
}

agent_logs() {
    docker logs brume-agent -f
}

main() {
    case $1 in
        start) start ;;
        stop) stop ;;
        rm) remove_jobs ;;
        ps) show_jobs ;;
        nuke) nuke ;;
        orch) orchestrator_logs ;;
        agent) agent_logs ;;
        *) echo "Usage: brm (brume cli) {start|stop|rm|ps|nuke|orch|agent}" ;;
    esac
}

main $1
