# This script takes care of testing your crate

set -ex

main() {
    local features="--features reqwest/native-tls-vendored"
    if [[ $TARGET =~ .*-freebsd ]]; then
        # N.B. still broken
        features="--features reqwest/rustls-tls"
    fi

    cross build $features --target $TARGET
    cross build $features --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test $features --target $TARGET
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
