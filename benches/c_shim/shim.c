#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include <ardupilotmega/mavlink.h>

/* Decodes `len` bytes one char at a time using the C reference parser
 * (c_library_v2) and returns the number of complete, framed messages produced.
 * Uses the buffer-based API so no per-channel static state is involved. */
int mavlink_codec_bench_count_messages(const uint8_t *data, size_t len) {
    mavlink_message_t rxmsg;
    mavlink_status_t status;
    mavlink_message_t r_message;
    mavlink_status_t r_status;

    memset(&rxmsg, 0, sizeof(rxmsg));
    memset(&status, 0, sizeof(status));

    int count = 0;
    for (size_t i = 0; i < len; i++) {
        if (mavlink_frame_char_buffer(&rxmsg, &status, data[i], &r_message,
                                      &r_status) == MAVLINK_FRAMING_OK) {
            count++;
        }
    }
    return count;
}
