package dev.hoilryu.hopentypeless.mobile;
import org.junit.Test;
import static org.junit.Assert.*;
import okhttp3.mockwebserver.MockWebServer;
import okhttp3.mockwebserver.MockResponse;
import okhttp3.mockwebserver.RecordedRequest;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.io.File;
import java.io.IOException;
import java.util.concurrent.TimeUnit;

public class ApiTest {
    private String address(MockWebServer server) {return "http://127.0.0.1:"+server.getPort();}
    @Test public void multipartContract() throws Exception {
        try(MockWebServer server=new MockWebServer()) {
            server.enqueue(new MockResponse().setBody("{\"raw_text\":\"hello\",\"polished_text\":\"Hello\"}"));server.start();
            File audio=File.createTempFile("dictation-test-",".wav");
            try {
                Files.write(audio.toPath(),"fake-audio-contract-test".getBytes(StandardCharsets.UTF_8));
                assertEquals("Hello",new Api.Call().run(address(server),audio).getString("polished_text"));
                RecordedRequest request=server.takeRequest(2,TimeUnit.SECONDS);assertNotNull(request);
                assertEquals("POST",request.getMethod());assertEquals("/api/dictate",request.getPath());
                assertNull(request.getHeader("Authorization"));
                assertEquals("android-v1",request.getHeader("X-H-OpenTypeless-Client"));
                assertTrue(request.getHeader("Content-Type").startsWith("multipart/form-data; boundary=Mobile"));
                String received=request.getBody().readUtf8();
                assertTrue(received.contains("name=\"file\"; filename=\"recording.wav\""));
                assertTrue(received.contains("Content-Type: audio/wav\r\n\r\nfake-audio-contract-test"));
            }finally{audio.delete();}
        }
    }
    @Test public void rejectRedirectAndHonorCancellation() throws Exception {
        try(MockWebServer server=new MockWebServer()) {
            server.enqueue(new MockResponse().setResponseCode(302).addHeader("Location","http://8.8.8.8/"));server.start();
            IOException redirect=assertThrows(IOException.class,()->new Api.Call().run(address(server),null));
            assertTrue(redirect.getMessage().contains("주소가 변경"));
            Api.Call canceled=new Api.Call();canceled.cancel();assertThrows(IOException.class,()->canceled.run(address(server),null));
            assertEquals(1,server.getRequestCount());
        }
    }
    @Test public void preserveActionableServerError() throws Exception {
        try(MockWebServer server=new MockWebServer()) {
            server.enqueue(new MockResponse().setResponseCode(429).setBody("{\"detail\":\"다른 녹음을 처리하고 있습니다.\"}"));server.start();
            IOException error=assertThrows(IOException.class,()->new Api.Call().run(address(server),null));
            assertEquals("다른 녹음을 처리하고 있습니다.",error.getMessage());
        }
    }
    @Test public void cancelPendingRequestThenRetry() throws Exception {
        java.util.concurrent.ExecutorService worker=java.util.concurrent.Executors.newSingleThreadExecutor();
        try(MockWebServer server=new MockWebServer()) {
            server.enqueue(new MockResponse().setSocketPolicy(okhttp3.mockwebserver.SocketPolicy.NO_RESPONSE));
            server.enqueue(new MockResponse().setBody("{\"status\":\"ok\"}"));server.start();
            Api.Call pending=new Api.Call();
            java.util.concurrent.Future<Boolean> stopped=worker.submit(()->{
                try {pending.run(address(server),null);return false;}catch(Exception expected){return true;}
            });
            assertNotNull(server.takeRequest(2,TimeUnit.SECONDS));
            pending.cancel();assertTrue(stopped.get(3,TimeUnit.SECONDS));
            assertEquals("ok",new Api.Call().run(address(server),null).getString("status"));
        } finally {worker.shutdownNow();}
    }
    @Test public void unavailableServerThenReconnect() throws Exception {
        int port;
        try(java.net.ServerSocket socket=new java.net.ServerSocket(0)){port=socket.getLocalPort();}
        String endpoint="http://127.0.0.1:"+port;
        assertThrows(IOException.class,()->new Api.Call().run(endpoint,null));
        try(MockWebServer server=new MockWebServer()) {
            server.enqueue(new MockResponse().setBody("{\"status\":\"ok\"}"));server.start(port);
            assertEquals("ok",new Api.Call().run(endpoint,null).getString("status"));
        }
    }

}
