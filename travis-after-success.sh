#!/usr/bin/env bash

set -e
set -x

if [ "${TRAVIS_PULL_REQUEST_BRANCH:-$TRAVIS_BRANCH}" != "master" ] && [ "$TRAVIS_RUST_VERSION" == "nightly" ]; then
    REMOTE_URL="$(git config --get remote.origin.url)";
    # Clone the repository fresh..for some reason checking out master fails
    # from a normal PR build's provided directory
    cd ${TRAVIS_BUILD_DIR}/.. && \
    git clone ${REMOTE_URL} "${TRAVIS_REPO_SLUG}-bench" && \
    cd  "${TRAVIS_REPO_SLUG}-bench" && \
    # Bench the pull request base or master
    git checkout "${TRAVIS_PULL_REQUEST_BRANCH:-master}" && \
    cargo bench | tee previous-benchmark && \
    # Bench the current commit that was pushed
    git checkout ${TRAVIS_COMMIT} && \
    cargo bench | tee current-benchmark && \
    cargo install cargo-benchcmp --force && \
    cargo benchcmp previous-benchmark current-benchmark;
fi
