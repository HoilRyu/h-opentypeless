package dev.hoilryu.hopentypeless.mobile;

import java.io.*;
import java.util.Arrays;

/** H records a canonical 44-byte-header WAV. Strip only a verified H WAV header. */
final class PcmAudio {
    static void decode(File source, File target) throws IOException {
        long length = source.length();
        if (length < 46 || length > Wav.MAX_PCM + 44L || (length - 44) % 2 != 0)
            throw new IOException("유효한 H PCM WAV가 아닙니다.");
        try (InputStream in = new BufferedInputStream(new FileInputStream(source))) {
            byte[] header = new byte[44];
            new DataInputStream(in).readFully(header);
            if (!Arrays.equals(header, Wav.header((int) length - 44)))
                throw new IOException("16kHz 모노 PCM16 H WAV만 지원합니다.");
            try (OutputStream out = new BufferedOutputStream(new FileOutputStream(target))) {
                byte[] buffer = new byte[32768];
                int count, total = 0;
                while ((count = in.read(buffer)) != -1) {
                    if (Thread.currentThread().isInterrupted()) throw new IOException("변환을 취소했습니다.");
                    total += count;
                    if (total > length - 44) throw new IOException("녹음 파일 크기가 변경되었습니다.");
                    out.write(buffer, 0, count);
                }
                if (total != length - 44) throw new IOException("녹음 파일이 잘렸습니다.");
            }
        } catch (IOException error) {
            target.delete();
            throw error;
        }
    }
}
