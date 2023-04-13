#!/bin/bash
set -e

if [ $# != 1 ]; then
  echo "Usage: bump_version.sh <new version>" 1>&2
  exit 1
fi

SCRIPT_DIR="$( pwd )"
cd $SCRIPT_DIR/..

OLD_VERSION="$(cat README.md |grep '^sailfish\s*=' |cut -d '"' -f2)"
NEW_VERSION="${1}"

echo "Bumping version (${OLD_VERSION} => ${NEW_VERSION})"

find . -name Cargo.toml -type f | while read f; do sed -i -e "0,/^version.*/{s/^version.*$/version = \"$NEW_VERSION\"/}" "$f"; done

# bump dependency version
find . -name Cargo.toml -type f | while read f; do sed -i -e "/^path = \"..\/sailfish.*\"/!b;n;cversion = \"$NEW_VERSION\"" "$f"; done

# bump version in README
sed -i -e "s/^\(sailfish.*\) = .*/\1 = \"$NEW_VERSION\"/" README.md

# bump version in documents
find docs/en -path "*.md" -type f |while read f; do sed -i -e "s/^\(sailfish.*\) = .*/\1 = \"$NEW_VERSION\"/" "$f"; done

cargo update -p sailfish --precise "${NEW_VERSION}"

#update the sailfish-compiler with cargo publish
#update the sailfish-macros with cargo publish
#update the sailfish with cargo publish