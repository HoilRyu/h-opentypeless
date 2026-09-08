package dev.hoilryu.hopentypeless.mobile;

import android.content.Context;
import android.content.SharedPreferences;

final class Config {
    private static SharedPreferences prefs(Context context) { return context.getSharedPreferences("connection", Context.MODE_PRIVATE); }
    static String server(Context context) { return prefs(context).getString("server", ""); }
    static void save(Context context, String server) throws Exception {
        String checkedServer = Policy.server(server);
        if (!prefs(context).edit().putString("server", checkedServer).remove("encrypted_token").remove("iv").commit())
            throw new Exception("설정을 저장하지 못했습니다.");
    }
}
