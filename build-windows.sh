# Download & extract the GCC 7.3.0 MinGW (SEH) - 64-bit version of SFML onto a folder called `windows`
# You will require the following dlls from `/usr/x86_64-w64-mingw32/bin/`:
# - libgcc_s_seh-1.dll
# - libstdc++6.dll
# And the following ones from SFML:
# - openal32.dll
# - sfml-window-2.dll
# - sfml-graphics-2.dll
# - sfml-system-2.dll
# - sfml-audio-2.dll
# Obviously this requires mingw-w64-gcc to be installed in your system

SFML_INCLUDE_DIR="$PWD/windows/SFML-2.5.1/include/" SFML_LIBS_DIR="$PWD/windows/SFML-2.5.1/lib/" cargo build --release --verbose --target x86_64-pc-windows-gnu