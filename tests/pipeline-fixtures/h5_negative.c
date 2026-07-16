int h5_not_missing(int value) { return value; }

__attribute__((visibility("hidden"))) int h5_hidden(int value) {
    return value + 1;
}

__attribute__((weak)) int h5_weak(int value) { return value + 2; }
