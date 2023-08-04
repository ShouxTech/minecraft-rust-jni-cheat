#include <Windows.h>

void create_console() {
    AllocConsole();
    freopen("CONIN$", "r", stdin);
    freopen("CONOUT$", "w", stdout);
    freopen("CONOUT$", "w", stderr);
}

void close_console() {
    FreeConsole();
}
