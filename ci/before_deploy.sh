# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          features="--features reqwest/native-tls-vendored" \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    cross rustc $features --bin so --target $TARGET --release -- -C lto

    cp target/$TARGET/release/so $stage/

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
