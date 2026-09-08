package dev.hoilryu.hopentypeless.mobile;

import org.json.JSONObject;
import java.io.*;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.util.UUID;

final class Api {
    static final class Call {
        private volatile boolean canceled;
        private volatile HttpURLConnection connection;
        void cancel() { canceled = true; HttpURLConnection c = connection; if (c != null) c.disconnect(); }
        private void check() throws IOException { if (canceled || Thread.currentThread().isInterrupted()) throw new IOException("요청을 취소했습니다."); }
        JSONObject run(String server, File audio) throws Exception {
            check();
            HttpURLConnection c = (HttpURLConnection) new URL(Policy.server(server) + (audio == null ? "/api/health" : "/api/dictate")).openConnection(Proxy.NO_PROXY);
            connection = c;
            try {
                check();
                c.setConnectTimeout(10000); c.setReadTimeout(420000); c.setInstanceFollowRedirects(false);
                c.setRequestProperty("Accept", "application/json");
                c.setRequestProperty("X-H-OpenTypeless-Client", "android-v1");
                if (audio != null) {
                    if (audio.length() == 0 || audio.length() > 4*1024*1024) throw new IOException("녹음 파일이 비어 있거나 너무 큽니다.");
                    String boundary = "Mobile" + UUID.randomUUID().toString().replace("-", "");
                    byte[] prefix = ("--"+boundary+"\r\nContent-Disposition: form-data; name=\"file\"; filename=\"recording.wav\"\r\nContent-Type: audio/wav\r\n\r\n").getBytes(StandardCharsets.UTF_8);
                    byte[] suffix = ("\r\n--"+boundary+"--\r\n").getBytes(StandardCharsets.UTF_8);
                    c.setRequestMethod("POST"); c.setDoOutput(true);
                    c.setRequestProperty("Content-Type", "multipart/form-data; boundary="+boundary);
                    c.setFixedLengthStreamingMode(prefix.length + audio.length() + suffix.length);
                    try (OutputStream out = c.getOutputStream(); InputStream in = new FileInputStream(audio)) {
                        out.write(prefix); byte[] bytes = new byte[16384]; int count;
                        while ((count = in.read(bytes)) != -1) { check(); out.write(bytes,0,count); }
                        out.write(suffix);
                    }
                } else c.setReadTimeout(10000);
                int code = c.getResponseCode(); check();
                if (code >= 300 && code < 400) throw new IOException("서버 주소가 변경되었습니다. 설정에서 직접 확인해 주세요.");
                InputStream body = code >= 400 ? c.getErrorStream() : c.getInputStream();
                if (body == null) throw new IOException("서버 응답 오류 ("+code+")");
                ByteArrayOutputStream output = new ByteArrayOutputStream();
                try (InputStream in = body) {
                    byte[] buffer = new byte[8192]; int count;
                    while ((count=in.read(buffer))!=-1) { check(); if(output.size()+count>1024*1024) throw new IOException("서버 응답이 너무 큽니다."); output.write(buffer,0,count); }
                }
                JSONObject result = new JSONObject(output.toString(StandardCharsets.UTF_8.name()));
                if (code != 200) throw new IOException(result.optString("detail", "서버 응답 오류 ("+code+")"));
                return result;
            } finally { c.disconnect(); connection = null; }
        }
    }
    static String message(Exception error) {
        if (error instanceof SocketTimeoutException) return "연결 시간이 초과되었습니다. H 앱과 네트워크 상태를 확인하세요.";
        if (error instanceof ConnectException || error instanceof NoRouteToHostException || error instanceof UnknownHostException) return "H 앱에 연결하지 못했습니다. 서버 주소와 Wi-Fi 또는 WireGuard 연결을 확인하세요.";
        if (error instanceof IOException || error instanceof IllegalArgumentException) return error.getMessage() == null ? "연결에 실패했습니다." : error.getMessage();
        return "처리에 실패했습니다. 서버 설정과 응답을 확인해 주세요.";
    }
}
