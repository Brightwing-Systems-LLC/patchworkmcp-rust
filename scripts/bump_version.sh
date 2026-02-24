#!/bin/bash
# Usage: ./scripts/bump_version.sh <version>
# Example: ./scripts/bump_version.sh 0.2.0
#
# Updates version in Cargo.toml, commits, and tags.

set -e

VERSION=${1}

if [[ -z "$VERSION" ]]; then
    echo "Error: Version is required"
    echo "Usage: $0 <version>"
    echo "Example: $0 0.2.0"
    exit 1
fi

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must match X.Y.Z format (e.g. 0.2.0)"
    exit 1
fi

echo "Bumping to version $VERSION..."

# Update Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

git add Cargo.toml
git commit -m "Bump version to $VERSION"
git tag "v$VERSION"

echo "Version bump completed successfully!"
echo "Don't forget to:"
echo "1. Update CHANGELOG.md"
echo "2. git add CHANGELOG.md && git commit -m 'Update changelog for version'"
echo "3. git push origin main && git push origin --tags"
