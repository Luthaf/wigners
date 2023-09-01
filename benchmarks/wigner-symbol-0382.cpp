#include "WignerSymbolSingleton.hpp"


extern "C" void ws0382_init(int l_max) {
    util::wigner_init(l_max, "Jmax", 3);
}

extern "C" double ws0382_wigner_3j(int two_j1, int two_j2, int two_j3, int two_m1, int two_m2, int two_m3) {
    return util::wigner_3j(two_j1, two_j2, two_j3, two_m1, two_m2, two_m3);
}
