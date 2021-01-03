# I still don't understand Make :( so I use this garbage...

name="plaatcraft"
version="0.1.0"
platform="x86_64"

rm -f -r .vscode

if [ "$1" == "clean" ]; then
    rm -r target

elif [ "$1" == "release" ]; then
    mkdir -p target/release

    for file in src/*.c; do
        gcc -Os -Iinclude -c $file -o target/release/$(basename $file .c).o
    done

    if [ "$(uname -s)" == "Linux" ]; then
        rm -f target/release/$name-v$version-$platform

        gcc -s $(find target/release -name *.o) -lglfw -lm -ldl -o target/release/$name-v$version-$platform
    else
        rm -f target/release/$name-v$version-$platform.exe

        gcc -s $(find target/release -name *.o) -static -lglfw3 -lm -lgdi32 -Wl,--subsystem,windows -o target/release/$name-v$version-$platform.exe
    fi

    rm -r target/release/*.o

    rm -f -r target/release/assets
    cp -r assets target/release

else
    mkdir -p target/debug

    for file in src/*.c; do
        object="target/debug/$(basename $file .c).o"
        if [[ $file -nt $object ]]; then
            if [[ $file == "src/glad.c" ]] || [[ $file == "src/stb_image.c" ]]; then
                gcc -Iinclude -c $file -o $object
                echo "Passed: $file"
            else
                if gcc -DDEBUG -Wall -Wextra -Wpedantic --std=c99 -Werror -Iinclude -c $file -o $object; then
                    echo "Passed: $file"
                else
                    echo "Failed: $file"
                    exit
                fi
            fi
        fi
    done

    if [ "$(uname -s)" == "Linux" ]; then
        rm -f target/debug/$name-v$version-$platform-debug

        gcc $(find target/debug -name *.o) -lglfw -lm -ldl -o target/debug/$name-v$version-$platform-debug

        ./target/debug/$name-v$version-$platform-debug
    else
        rm -f target/debug/$name-v$version-$platform-debug.exe

        gcc $(find target/debug -name *.o) -lglfw3 -lm -o target/debug/$name-v$version-$platform-debug.exe

        ./target/debug/$name-v$version-$platform-debug.exe
    fi
fi
