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
    private final ExecutorService finalizer = Executors.newSingleThreadExecutor();
    private final Runnable limit = ()->stopAndSend();
    private PcmRecorder recorder;
    private File recording, finishingAudio;
    private Api.Call call;
    private SystemStt systemStt;
    private LocalJob localJob;
    private File localAudio;
    private boolean systemListening,localRecording;
    private String activeStt,activeLlm,activePrompt;
    private long generation;
    private boolean processing, password;
    private KeyboardPanel panel;
    private long recordingStarted;
    private final Runnable meter = new Runnable() {
        @Override public void run() {
            if (!isRecording() || panel==null) return;
            try {if(recorder==null&&systemStt!=null&&systemStt.streamed())panel.recordingLevel((android.os.SystemClock.elapsedRealtime()-recordingStarted)/1000,systemStt.amplitude());else if(recorder==null)panel.recordingTime((android.os.SystemClock.elapsedRealtime()-recordingStarted)/1000);else panel.recordingLevel((android.os.SystemClock.elapsedRealtime()-recordingStarted)/1000,recorder.getMaxAmplitude());}
            catch(RuntimeException ignored) {}
            main.postDelayed(this,100);
        }
    };
    private String rawText="", polishedText="";

    @Override public void onCreate() {
        super.onCreate();
        File[] stale=getCacheDir().listFiles((dir,name)->name.startsWith("dictation-")&&(name.endsWith(".wav")||name.endsWith(".pcm")));
        if(stale!=null)for(File file:stale)file.delete();
    }
    @Override public View onCreateInputView() {
        panel=new KeyboardPanel(this,new KeyboardPanel.Actions(){
            public void mode(){toggleMode();}
            public void record(){if(processing){reset();show("취소했습니다.");}else if(isRecording())stopAndSend();else startRecording();}
            public void insert(String value){DictationIme.this.insert(value);}
            public void clear(){boolean active=isRecording();reset();show(active?"녹음을 취소했습니다.":"결과를 지웠습니다.");}
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
    @Override public void onStartInputView(EditorInfo info,boolean restarting){super.onStartInputView(info,restarting);refresh();if(password)show("비밀번호 입력칸에서는 녹음을 사용할 수 없습니다.");else if(!LocalConfig.local(this)||Capability.selected(this)==null)show("최대 2분 · 확인 후 직접 삽입");}
    @Override public void onFinishInputView(boolean finishingInput){reset();super.onFinishInputView(finishingInput);}
    @Override public void onFinishInput(){reset();super.onFinishInput();}
    @Override public void onWindowHidden(){reset();super.onWindowHidden();}
    @Override public void onDestroy(){reset();worker.shutdownNow();finalizer.shutdown();super.onDestroy();}

    private boolean isRecording(){return recorder!=null||systemListening;}
    private void toggleMode(){
        if(processing||isRecording())return;
        reset();LocalConfig.local(this,!LocalConfig.local(this));refresh();
        String reason=LocalConfig.local(this)?Capability.selected(this):null;
        show(reason==null?LocalConfig.label(this)+" · 설정에서 STT·교정 모델 선택":reason);
    }
    private void show(String message){if(panel!=null)panel.message(message);}
    private void refresh(){if(panel!=null){panel.render(rawText,polishedText,isRecording(),processing,password);panel.mode(LocalConfig.label(this),isRecording()||processing);panel.availability(LocalConfig.local(this)?Capability.selected(this):null,!isRecording()&&!processing&&!password);}}
    private void startRecording(){
        if(password||processing||getCurrentInputConnection()==null)return;
        if(checkSelfPermission(Manifest.permission.RECORD_AUDIO)!=PackageManager.PERMISSION_GRANTED){show("연결 설정에서 마이크 권한을 먼저 허용해 주세요.");return;}
        reset();
        try {
            localRecording=LocalConfig.local(this);
            if(localRecording){
                String reason=Capability.selected(this);if(reason!=null){show(reason);return;}
                activeStt=LocalConfig.stt(this);activeLlm=LocalConfig.llm(this);activePrompt=LocalConfig.prompt(this);
                if(activeStt.equals("system")){startSystem();return;}
            }else Policy.server(Config.server(this));
            recording=File.createTempFile("dictation-",".wav",getCacheDir());
            recorder = new PcmRecorder(this,recording);
            recorder.start();recordingStarted=android.os.SystemClock.elapsedRealtime();main.post(meter);main.postDelayed(limit,119000);refresh();show("녹음 중… 끝나면 종료 버튼을 누르세요.");
        } catch(Exception error){reset();show(error instanceof IllegalArgumentException?error.getMessage():"녹음을 시작하지 못했습니다. 연결 설정과 마이크 권한을 확인하세요.");}
    }
    private void stopAndSend(){
        if(systemStt!=null&&systemListening){
            main.removeCallbacks(limit);systemListening=false;processing=true;refresh();show("기본 STT 결과를 기다리는 중…");systemStt.stop();
            final long current=generation;main.postDelayed(()->{if(current==generation&&systemStt!=null){reset();show("기본 STT 응답 시간이 초과되었습니다.");}},15000);return;
        }
        if(recorder==null)return;
        main.removeCallbacks(limit);main.removeCallbacks(meter);PcmRecorder previous=recorder;recorder=null;
        final File audio=recording;recording=null;finishingAudio=audio;
        final long current=generation;
        previous.requestStop();
        processing=true;refresh();show("녹음을 마무리하고 있습니다…");
        finalizer.execute(()->{
            String error=null;
            try {previous.stop();} catch(RuntimeException failure){error="녹음을 마무리하지 못했습니다. 마이크를 확인하고 다시 시도하세요.";}
            finally {previous.release();}
            final String message=error;
            if(message!=null)audio.delete();
            main.post(()->{
                if(current!=generation){audio.delete();return;}
                finishingAudio=null;
                if(message!=null){reset();show(message);return;}
                sendRecording(audio,current);
            });
        });
    }
    private void sendRecording(File audio,long current){
        if(localRecording){localAudio=audio;runLocal("",0,audio);return;}
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
    private void startSystem(){
        final long current=generation;recordingStarted=android.os.SystemClock.elapsedRealtime();
        systemStt=new SystemStt(this,new SystemStt.Callback(){
            public void level(float value){if(current==generation&&panel!=null)panel.recordingLevel((android.os.SystemClock.elapsedRealtime()-recordingStarted)/1000,(int)(Math.max(0,value)*1200));}
            public void result(String text,long ms){if(current!=generation)return;main.removeCallbacks(limit);systemStt.cancel();systemStt=null;systemListening=false;runLocal(text,ms,null);}
            public void error(String error){if(current!=generation)return;String saved=systemStt==null?"":systemStt.text();reset();rawText=Policy.insertion(saved);polishedText=rawText;refresh();show(error);}
        });
        systemListening=true;refresh();show("기본 온디바이스 STT · 쉬어도 계속 듣습니다. 끝나면 종료 버튼을 누르세요.");systemStt.start();main.post(meter);
    }
    private void runLocal(String text,long asrMs,File audio){
        final long current=generation;
        processing=true;refresh();show(activeStt.equals("system")?"휴대폰에서 교정 중…":"Whisper로 음성 인식 중…");
        android.os.Bundle args=new android.os.Bundle();args.putString("stt",activeStt);args.putString("llm",activeLlm);args.putString("prompt",activePrompt);args.putString("raw",text);args.putLong("asr_ms",asrMs);if(audio!=null)args.putString("audio",audio.getAbsolutePath());
        localJob=new LocalJob(this,args,new LocalJob.Callback(){
            public void stage(android.os.Bundle result){if(current==generation)show(activeLlm.equals("none")?"인식 완료":"Gemma 4로 교정 중…");}
            public void done(android.os.Bundle result){
                if(audio!=null)audio.delete();if(current!=generation)return;localJob=null;localAudio=null;processing=false;
                rawText=Policy.insertion(result.getString("raw",""));polishedText=Policy.insertion(result.getString("polished",rawText));refresh();
                String error=result.getString("error");
                LocalConfig.recordModelResult(DictationIme.this,result);refresh();
                String metrics=String.format(java.util.Locale.ROOT,"%s · 인식 %.1f초 · 교정 %.1f초",activeStt,result.getLong("asr_ms")/1000.0,result.getLong("polish_ms")/1000.0);
                LocalConfig.select(DictationIme.this,"last-result",metrics+" · 모델 로딩 "+(result.getLong("stt_load_ms")+result.getLong("llm_load_ms"))/1000.0+"초 · "+activeLlm+" · PSS 최대 "+result.getLong("peak_pss_kb")/1024+"MB"+(error==null?"":" · "+error));
                show(error!=null?error+(rawText.isEmpty()?"":" · 인식 원문을 사용할 수 있습니다."):rawText.isEmpty()?"인식된 말이 없습니다.":metrics+" · 확인 후 삽입");
            }
        });localJob.start();
    }
    private void editKey(int code){
        if(processing||isRecording())return;
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
        if(password||processing||isRecording()||text.isEmpty())return;
        InputConnection connection=getCurrentInputConnection();
        if(connection==null){reset();return;}
        // Dictation inserts text only. Editing keys are sent exclusively on explicit taps.
        if(connection.commitText(Policy.insertion(text),1)){reset();show("삽입했습니다. 명령 실행은 직접 해주세요.");}
        else show("입력칸에 넣지 못했습니다. 입력칸을 다시 선택하세요.");
    }
    private void reset(){
        if(panel!=null)panel.stopRepeating();
        generation++;main.removeCallbacksAndMessages(null);
        if(systemStt!=null){systemStt.cancel();systemStt=null;}systemListening=false;
        if(localJob!=null){localJob.cancel();localJob=null;}
        if(localAudio!=null){localAudio.delete();localAudio=null;}
        if(recorder!=null){recorder.cancel();recorder=null;}
        if(recording!=null){recording.delete();recording=null;}
        if(finishingAudio!=null){finishingAudio.delete();finishingAudio=null;}
        if(call!=null){call.cancel();call=null;}
        processing=false;rawText="";polishedText="";refresh();
    }
}
