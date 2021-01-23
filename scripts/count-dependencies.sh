#!/bin/bash

IFS=$'\n'
PACKAGES=("sailfish" "sailfish-macros" "sailfish-compiler")

git-root () {
    if git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
        cd `git rev-parse --show-toplevel`;
    fi
}

get_dependencies() {
    cargo tree -p "$1" | while read line; do
        dev_dependencies_re="\[dev-dependencies\]"
        crate_re="[a-zA-Z0-9_-]+ v[^ ]+"

        if [[ "$line" =~ $crate_re ]]; then
            echo ${BASH_REMATCH[0]}
            continue
        fi

        if [[ "$line" =~ $dev_dependencies_re ]]; then
            break
        fi
    done
}

remove_packages() {
    local found

    for dep in $@; do
        found=0
        for pkg in ${PACKAGES[@]}; do
            pat="$pkg v[^ ]"
            if [[ "$dep" =~ $pat ]]; then
                found=1
                break
            fi
        done

        if [[ $found == 0 ]]; then
            echo $dep
        fi
    done
}

# go to root directory
cd `git rev-parse --show-toplevel`

deps=()

for pkg in ${PACKAGES[@]}; do
    deps+=( `get_dependencies $pkg` )
done

deps=( $(printf "%s\n" "${deps[@]}" | sort -u) )

remove_packages ${deps[*]} |wc -l
