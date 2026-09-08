package dev.hoilryu.hopentypeless.mobile;
import org.junit.Test;
import static org.junit.Assert.*;
public class PolicyTest {
    @Test public void privateHttpAndHttps() {
        assertEquals("http://192.168.0.123:8787",Policy.server(" http://192.168.0.123:8787/ "));
        assertEquals("http://10.1.0.4:8787",Policy.server("http://10.1.0.4:8787"));
        assertEquals("https://example.com",Policy.server("https://example.com"));
    }
    @Test public void rejectCredentialLeakDestinations() {
        for(String value:new String[]{"http://example.com","http://192.168.0.123.evil.com","http://user@192.168.0.1","http://192.168.0.1/path","http://192.168.0.1?key=abc","http://8.8.8.8","http://172.32.0.1","http://192.168.0.999","file:///tmp/x","http://192.168.0.1:0"})
            assertThrows(value,IllegalArgumentException.class,()->Policy.server(value));
    }
    @Test public void preventTerminalControlAndEnter() {
        assertEquals("echo hello  rm file [31m",Policy.insertion("echo hello\r\nrm file\u001b[31m\n"));
        assertEquals("한글 😀 42",Policy.insertion("한글 😀 42"));
        assertEquals("a b",Policy.insertion("a\u2028b"));
    }
}
