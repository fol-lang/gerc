#include <stdint.h>

struct parc_packet {
    int value;
};

struct parc_packet h4_packet_roundtrip(struct parc_packet packet) {
    packet.value += 37;
    return packet;
}

uint32_t h4_mode_roundtrip(uint32_t mode) {
    return mode ^ UINT32_C(0x5a5aa5a5);
}
