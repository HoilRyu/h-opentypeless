package dev.hoilryu.hopentypeless.mobile;

import android.content.Context;
import android.os.Handler;
import android.os.Looper;
import android.view.MotionEvent;
import android.widget.ImageButton;

final class RepeatKeyButton extends ImageButton {
    private final Handler handler=new Handler(Looper.getMainLooper());
    private final PressRepeater repeater=new PressRepeater(new PressRepeater.Scheduler(){
        public void post(Runnable task,long delay){handler.postDelayed(task,delay);}
        public void remove(Runnable task){handler.removeCallbacks(task);}
    },()->performClick());
    RepeatKeyButton(Context context){super(context);}
    void stopRepeating(){repeater.stop();setPressed(false);}
    @Override public boolean onTouchEvent(MotionEvent event){
        if(!isEnabled()){stopRepeating();return false;}
        switch(event.getActionMasked()){
            case MotionEvent.ACTION_DOWN:setPressed(true);getParent().requestDisallowInterceptTouchEvent(true);repeater.start();return true;
            case MotionEvent.ACTION_MOVE:
                if(event.getX()<0||event.getY()<0||event.getX()>getWidth()||event.getY()>getHeight())stopRepeating();return true;
            case MotionEvent.ACTION_UP:case MotionEvent.ACTION_CANCEL:stopRepeating();return true;
            default:return true;
        }
    }
    @Override public boolean performClick(){super.performClick();return true;}
    @Override public void setEnabled(boolean enabled){if(!enabled&&repeater!=null)stopRepeating();super.setEnabled(enabled);}
    @Override protected void onDetachedFromWindow(){stopRepeating();super.onDetachedFromWindow();}
    @Override public void onWindowFocusChanged(boolean focused){if(!focused&&repeater!=null)stopRepeating();super.onWindowFocusChanged(focused);}
}
