if [ -z ${TRAVIS_OS_NAME+x} ]; then
    case `uname -s` in
        Linux)
            TRAVIS_OS_NAME=linux
            ;;
        Darwin)
            TRAVIS_OS_NAME=osx
            ;;
        *)
            echo "Unknown OS"
            exit 1
    esac
fi

if [ -z ${TRAVIS_BRANCH+x} ]; then
    TRAVIS_BRANCH=$(git rev-parse --abbrev-ref HEAD)
fi

PYTHON=python3
