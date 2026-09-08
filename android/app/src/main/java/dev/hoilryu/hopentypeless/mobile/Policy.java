package dev.hoilryu.hopentypeless.mobile;

import java.net.URI;

/** Shared input boundaries; never resolve a hostname to authorize plain HTTP. */
final class Policy {
    static String server(String input) {
        try {
            URI uri = new URI(input.trim());
            String scheme = uri.getScheme(), host = uri.getHost();
            if (host == null || uri.getRawUserInfo() != null || uri.getRawQuery() != null || uri.getRawFragment() != null
                    || !(uri.getPath().isEmpty() || uri.getPath().equals("/"))
                    || uri.getPort() < -1 || uri.getPort() == 0 || uri.getPort() > 65535) throw new Exception();
            if (!"https".equals(scheme) && !("http".equals(scheme) && privateV4(host))) throw new Exception();
            return new URI(scheme, null, host, uri.getPort(), null, null, null).toString();
        } catch (Exception error) {
            throw new IllegalArgumentException("서버 주소를 확인하세요. HTTP는 내부 IPv4 주소만 지원합니다. 예: http://192.168.0.123:8787");
        }
    }
    private static boolean privateV4(String host) {
        String[] parts = host.split("\\.", -1);
        if (parts.length != 4) return false;
        int[] p = new int[4];
        for (int i=0;i<4;i++) {
            if (!parts[i].matches("0|[1-9][0-9]{0,2}")) return false;
            p[i] = Integer.parseInt(parts[i]);
            if (p[i] > 255) return false;
        }
        return p[0] == 10 || p[0] == 127 || (p[0] == 192 && p[1] == 168) || (p[0] == 172 && p[1] >= 16 && p[1] <= 31);
    }
    static String insertion(String text) {
        // Terminal input must never contain Enter, Escape, or other control sequences.
        StringBuilder result = new StringBuilder();
        text.codePoints().forEach(cp -> {
            if (Character.isISOControl(cp) || cp == 0x2028 || cp == 0x2029) result.append(' ');
            else result.appendCodePoint(cp);
        });
        return result.toString().trim();
    }
}
