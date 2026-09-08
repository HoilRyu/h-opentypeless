package dev.hoilryu.hopentypeless.mobile;

import android.Manifest;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.inputmethodservice.InputMethodService;
import android.os.Handler;
import android.os.Looper;
import android.text.InputType;
import android.view.View;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputConnection;
import android.view.inputmethod.InputMethodManager;
import android.widget.*;
import org.json.JSONObject;
import java.io.File;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

public final class DictationIme extends InputMethodService {
    private final Handler main = new Handler(Looper.getMainLooper());
    private final ExecutorService worker = Executors.newSingleThreadExecutor();
    private final Runnable limit = ()->stopAndSend();
    private PcmRecorder recorder;
    private File recording;
    private Api.Call call;
    private long generation;
    private boolean processing, password;
    private KeyboardPanel panel;
    private long recordingStarted;
    private final Runnable meter = new Runnable() {
        @Override public void run() {
            if (recorder==null || panel==null) return;
            try {panel.recordingLevel((android.os.SystemClock.elapsedRealtime()-recordingStarted)/1000,recorder.getMaxAmplitude());}
            catch(RuntimeException ignored) {}
            main.postDelayed(this,100);
        }
    };
    private String rawText="", polishedText="";

    @Override public void onCreate() {
        super.onCreate();
        File[] stale=getCacheDir().listFiles((dir,name)->name.startsWith("dictation-")&&name.endsWith(".wav"));
        if(stale!=null)for(File file:stale)file.delete();
    }
    @Override public View onCreateInputView() {
        panel=new KeyboardPanel(this,new KeyboardPanel.Actions(){
            public void record(){if(processing){reset();show("취소했습니다.");}else if(recorder!=null)stopAndSend();else startRecording();}
            public void insert(String value){DictationIme.this.insert(value);}
            public void clear(){boolean active=recorder!=null;reset();show(active?"녹음을 취소했습니다.":"결과를 지웠습니다.");}
            public void keyboard(){reset();((InputMethodManager)getSystemService(INPUT_METHOD_SERVICE)).showInputMethodPicker();}
            public void moveLeft(){editKey(android.view.KeyEvent.KEYCODE_DPAD_LEFT);}
            public void moveRight(){editKey(android.view.KeyEvent.KEYCODE_DPAD_RIGHT);}
            public void backspace(){editKey(android.view.KeyEvent.KEYCODE_DEL);}
            public void settings(){reset();startActivity(new Intent(DictationIme.this,SettingsActivity.class).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK));}
        });
        refresh();return panel;
    }
    @Override public boolean onEvaluateFullscreenMode(){return false;}
    @Override public void onStartInput(EditorInfo info,boolean restarting){
        super.onStartInput(info,restarting);reset();
        int klass=info.inputType&InputType.TYPE_MASK_CLASS,variation=info.inputType&InputType.TYPE_MASK_VARIATION;
        password=(klass==InputType.TYPE_CLASS_TEXT&&(variation==InputType.TYPE_TEXT_VARIATION_PASSWORD||variation==InputType.TYPE_TEXT_VARIATION_WEB_PASSWORD||variation==InputType.TYPE_TEXT_VARIATION_VISIBLE_PASSWORD))
            ||(klass==InputType.TYPE_CLASS_NUMBER&&variation==InputType.TYPE_NUMBER_VARIATION_PASSWORD);
        refresh();if(password)show("비밀번호 입력칸에서는 녹음을 사용할 수 없습니다.");
    }
    @Override public void onStartInputView(EditorInfo info,boolean restarting){super.onStartInputView(info,restarting);refresh();show(password?"비밀번호 입력칸에서는 녹음을 사용할 수 없습니다.":"최대 2분 · 확인 후 직접 삽입");}
    @Override public void onFinishInputView(boolean finishingInput){reset();super.onFinishInputView(finishingInput);}
    @Override public void onFinishInput(){reset();super.onFinishInput();}
    @Override public void onWindowHidden(){reset();super.onWindowHidden();}
    @Override public void onDestroy(){reset();worker.shutdownNow();super.onDestroy();}

    private void show(String message){if(panel!=null)panel.message(message);}
    private void refresh(){if(panel!=null)panel.render(rawText,polishedText,recorder!=null,processing,password);}
    private void startRecording(){
        if(password||processing||getCurrentInputConnection()==null)return;
        if(checkSelfPermission(Manifest.permission.RECORD_AUDIO)!=PackageManager.PERMISSION_GRANTED){show("연결 설정에서 마이크 권한을 먼저 허용해 주세요.");return;}
        reset();
        try {
            Policy.server(Config.server(this));
            recording=File.createTempFile("dictation-",".wav",getCacheDir());
            recorder = new PcmRecorder(this,recording);
            recorder.start();recordingStarted=android.os.SystemClock.elapsedRealtime();main.post(meter);main.postDelayed(limit,119000);refresh();show("녹음 중… 끝나면 종료 버튼을 누르세요.");
        } catch(Exception error){reset();show(error instanceof IllegalArgumentException?error.getMessage():"녹음을 시작하지 못했습니다. 연결 설정과 마이크 권한을 확인하세요.");}
    }
    private void stopAndSend(){
        if(recorder==null)return;
        main.removeCallbacks(limit);main.removeCallbacks(meter);PcmRecorder previous=recorder;recorder=null;
        try { previous.stop(); } catch(RuntimeException error){previous.release();reset();show("녹음이 너무 짧거나 마이크를 사용할 수 없습니다. 다시 녹음하세요.");return;}
        previous.release();
        final File audio=recording;recording=null;
        final long current=generation;
        final String server;
        try {server=Policy.server(Config.server(this));}
        catch(Exception error){audio.delete();reset();show("연결 설정을 다시 저장해 주세요.");return;}
        processing=true;refresh();show("H 앱에서 음성 인식·교정 중…");
        final Api.Call request=new Api.Call();call=request;
        // Full-request deadline also covers stalled socket writes.
        Runnable deadline=()->{if(current==generation&&processing){reset();show("처리 시간이 초과되었습니다. 다시 시도하세요.");}};
        main.postDelayed(deadline,420000);
        worker.execute(()->{
            String original="",result="",message;
            try {
                JSONObject response=request.run(server,audio);
                original=Policy.insertion(response.getString("raw_text"));result=Policy.insertion(response.getString("polished_text"));
                message=response.isNull("warning")?"내용을 확인하고 삽입하세요 · 줄바꿈은 공백으로 변환":response.getString("warning");
                if(original.isEmpty()&&result.isEmpty())message="인식된 말이 없습니다. 다시 녹음해 주세요.";
            } catch(Exception error){message=Api.message(error);}
            finally{audio.delete();}
            String finalRaw=original,finalPolished=result,finalMessage=message;
            main.post(()->{
                main.removeCallbacks(deadline);
                if(current!=generation)return;
                call=null;processing=false;rawText=finalRaw;polishedText=finalPolished;refresh();show(finalMessage);
            });
        });
    }
    private void editKey(int code){
        if(processing||recorder!=null)return;
        if(code!=android.view.KeyEvent.KEYCODE_DPAD_LEFT&&code!=android.view.KeyEvent.KEYCODE_DPAD_RIGHT&&code!=android.view.KeyEvent.KEYCODE_DEL)return;
        InputConnection connection=getCurrentInputConnection();if(connection==null)return;
        long now=android.os.SystemClock.uptimeMillis();
        // Hardware-compatible arrows/delete also work with terminal InputConnections.
        int flags=android.view.KeyEvent.FLAG_SOFT_KEYBOARD|android.view.KeyEvent.FLAG_KEEP_TOUCH_MODE;
        boolean handled=EditingKeys.dispatch(code,(action,key)->connection.sendKeyEvent(
            new android.view.KeyEvent(now,android.os.SystemClock.uptimeMillis(),action,key,0,0,android.view.KeyCharacterMap.VIRTUAL_KEYBOARD,0,flags)));
        if(!handled)show("이 입력칸에서 편집 키를 처리하지 못했습니다.");
    }
    private void insert(String text){
        if(password||processing||recorder!=null||text.isEmpty())return;
        InputConnection connection=getCurrentInputConnection();
        if(connection==null){reset();return;}
        // Dictation inserts text only. Editing keys are sent exclusively on explicit taps.
        if(connection.commitText(Policy.insertion(text),1)){reset();show("삽입했습니다. 명령 실행은 직접 해주세요.");}
        else show("입력칸에 넣지 못했습니다. 입력칸을 다시 선택하세요.");
    }
    private void reset(){
        if(panel!=null)panel.stopRepeating();
        generation++;main.removeCallbacksAndMessages(null);
        if(recorder!=null){try{recorder.reset();}catch(RuntimeException ignored){}recorder.release();recorder=null;}
        if(recording!=null){recording.delete();recording=null;}
        if(call!=null){call.cancel();call=null;}
        processing=false;rawText="";polishedText="";refresh();
    }
}
