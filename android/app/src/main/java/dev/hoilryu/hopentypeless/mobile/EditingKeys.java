package dev.hoilryu.hopentypeless.mobile;

import android.view.KeyEvent;

/** Dispatch only horizontal movement and Backspace, with balanced press/release. */
final class EditingKeys {
    interface Sink { boolean send(int action,int code); }
    static boolean dispatch(int code,Sink sink){
        if(code!=KeyEvent.KEYCODE_DPAD_LEFT&&code!=KeyEvent.KEYCODE_DPAD_RIGHT&&code!=KeyEvent.KEYCODE_DEL)return false;
        boolean down=sink.send(KeyEvent.ACTION_DOWN,code);
        boolean up=sink.send(KeyEvent.ACTION_UP,code);
        return down&&up;
    }
}
