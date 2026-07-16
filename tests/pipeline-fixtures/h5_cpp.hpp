class h5_cpp_type {
public:
    virtual ~h5_cpp_type();
    int value;
};

extern "C" int h5_cpp_bridge(h5_cpp_type *value);
