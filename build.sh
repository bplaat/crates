# I still don't understand Make :( so I use this garbage...

name="plaatcraft"
version="0.1.0"
platform="x86_64"

rm -f -r .vscode

if [ "$1" == "clean" ]; then
    rm -r target

elif [ "$1" == "release" ]; then
    mkdir -p target/release

    for file in $(find src -name "*.c"); do
        folder=$(dirname ${file:4:-2})
        if [[ "$folder" != "." ]]; then
            mkdir -p "target/release/$folder"
        fi

        if gcc -Os -Iinclude -c $file -o "target/release/${file:4:-2}.o"; then
            echo "Passed: ${file:4}"
        else
            echo "Failed: ${file:4}"
            exit
        fi
    done

    if [ "$(uname -s)" == "Linux" ]; then
        rm -f target/release/$name-v$version-$platform

        gcc -s $(find target/release -name *.o) -lglfw -lm -ldl -o target/release/$name-v$version-$platform

        find target/release -not -name "$name-v$version-$platform" -delete
    else
        rm -f target/release/$name-v$version-$platform.exe

        gcc -s $(find target/release -name *.o) -static -lglfw3 -lm -lgdi32 -Wl,--subsystem,windows -o target/release/$name-v$version-$platform.exe

        find target/release -not -name "$name-v$version-$platform.exe" -delete
    fi

    rm -f -r target/release/assets
    cp -r assets target/release

else
    clear

    mkdir -p target/debug

    for file in $(find src -name "*.c"); do
        object="target/debug/${file:4:-2}.o"

        folder=$(dirname ${file:4:-2})
        if [[ "$folder" != "." ]]; then
            mkdir -p "target/debug/$folder"
        fi

        if [[ $file -nt $object ]]; then
            rm -f $object

            if [[ $file == "src/glad/glad.c" ]] || [[ $file == "src/textures/stb_image.c" ]]; then
                if gcc -Iinclude -c $file -o $object; then
                    echo "Passed: ${file:4}"
                else
                    echo "Failed: ${file:4}"
                    exit
                fi
            else
                if gcc -DDEBUG -Wall -Wextra -Wpedantic --std=c11 -Werror -Iinclude -c $file -o $object; then
                    echo "Passed: ${file:4}"
                else
                    echo "Failed: ${file:4}"
                    exit
                fi
            fi
        fi
    done

    if [ "$(uname -s)" == "Linux" ]; then
        rm -f target/debug/$name-v$version-$platform-debug

        gcc $(find target/debug -name *.o) -lglfw -lm -ldl -o target/debug/$name-v$version-$platform-debug

        if [ "$1" == "debug" ]; then
            gdb ./target/debug/$name-v$version-$platform-debug
        else
            ./target/debug/$name-v$version-$platform-debug
        fi
    else
        rm -f target/debug/$name-v$version-$platform-debug.exe

        gcc $(find target/debug -name *.o) -lglfw3 -lm -o target/debug/$name-v$version-$platform-debug.exe

        if [ "$1" == "debug" ]; then
            gdb ./target/debug/$name-v$version-$platform-debug.exe
        else
            ./target/debug/$name-v$version-$platform-debug.exe
        fi
    fi
fi
