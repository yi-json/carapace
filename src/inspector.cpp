#include <iostream>
#include <sys/utsname.h>

// extern "C" prevents C++ name mangling so Rust can find this function
extern "C" {
    void inspect_system() {
        struct utsname buffer;
        if (uname(&buffer) != 0) {
            std::cerr << "[C++] Error: Could not read kernel version." << std::endl;
            return;
        }

        std::cout << "\n[C++] --- Carapace Container Runtime ---" << std::endl;
        std::cout << "[C++] Kernel: " << buffer.sysname << " " << buffer.release << std::endl;
        std::cout << "[C++] Arch:   " << buffer.machine << std::endl;
        std::cout << "[C++] Status: Systems Operational" << std::endl;
        std::cout << "--------------------------------------\n" << std::endl;
    }
}