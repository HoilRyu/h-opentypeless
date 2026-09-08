package dev.hoilryu.hopentypeless.mobile;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
final class Wav {
    static final int MAX_PCM=16000*2*120;
    static byte[] header(int bytes) {
        if(bytes<0||bytes>MAX_PCM||bytes%2!=0)throw new IllegalArgumentException("Invalid PCM size");
        ByteBuffer b=ByteBuffer.allocate(44).order(ByteOrder.LITTLE_ENDIAN);
        b.put("RIFF".getBytes(StandardCharsets.US_ASCII)).putInt(bytes+36).put("WAVEfmt ".getBytes(StandardCharsets.US_ASCII));
        b.putInt(16).putShort((short)1).putShort((short)1).putInt(16000).putInt(32000).putShort((short)2).putShort((short)16);
        b.put("data".getBytes(StandardCharsets.US_ASCII)).putInt(bytes);return b.array();
    }
}
