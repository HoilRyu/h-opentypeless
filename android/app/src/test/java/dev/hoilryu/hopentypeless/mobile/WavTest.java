package dev.hoilryu.hopentypeless.mobile;
import org.junit.Test;
import static org.junit.Assert.*;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
public class WavTest {
    @Test public void encodesCanonicalProviderFormat(){
        byte[] h=Wav.header(32000);ByteBuffer b=ByteBuffer.wrap(h).order(ByteOrder.LITTLE_ENDIAN);
        assertEquals(44,h.length);assertEquals(32036,b.getInt(4));assertEquals(16000,b.getInt(24));assertEquals(32000,b.getInt(28));assertEquals(1,b.getShort(22));assertEquals(16,b.getShort(34));assertEquals(32000,b.getInt(40));
    }
    @Test public void rejectsInvalidSizes(){for(int n:new int[]{-1,1,Wav.MAX_PCM+2}){try{Wav.header(n);fail();}catch(IllegalArgumentException expected){}}}
}
