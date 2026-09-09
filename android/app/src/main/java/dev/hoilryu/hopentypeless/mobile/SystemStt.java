package dev.hoilryu.hopentypeless.mobile;

import android.content.*;
import android.os.*;
import android.speech.*;
import java.util.ArrayList;

final class SystemStt {
    interface Callback {void level(float value);void result(String text,long asrMs);void error(String error);}
    private final Context context;
    private final Callback callback;
    private final Handler main=new Handler(Looper.getMainLooper());
    private final SpeechSession session;
    private SpeechRecognizer recognizer;
    private RecordingLease lease;
    private SttAudioStream audioStream;
    private boolean closed;
    private long stopped;
    private int starts,segments,levelEvents;
    static Intent intent(){return new Intent(RecognizerIntent.ACTION_RECOGNIZE_SPEECH).putExtra(RecognizerIntent.EXTRA_LANGUAGE_MODEL,RecognizerIntent.LANGUAGE_MODEL_FREE_FORM).putExtra(RecognizerIntent.EXTRA_LANGUAGE,"ko-KR").putExtra(RecognizerIntent.EXTRA_PARTIAL_RESULTS,true);}
    static Intent listeningIntent(){return intent();}
    static Intent streamIntent(ParcelFileDescriptor input){if(Build.VERSION.SDK_INT<33)throw new IllegalStateException("Android 13 이상 필요");return intent().putExtra(RecognizerIntent.EXTRA_AUDIO_SOURCE,input).putExtra(RecognizerIntent.EXTRA_AUDIO_SOURCE_CHANNEL_COUNT,1).putExtra(RecognizerIntent.EXTRA_AUDIO_SOURCE_ENCODING,android.media.AudioFormat.ENCODING_PCM_16BIT).putExtra(RecognizerIntent.EXTRA_AUDIO_SOURCE_SAMPLING_RATE,16000).putExtra(RecognizerIntent.EXTRA_SEGMENTED_SESSION,RecognizerIntent.EXTRA_AUDIO_SOURCE);}
    boolean streamed(){return Build.VERSION.SDK_INT>=33;}
    int amplitude(){if(audioStream==null)return 0;levelEvents++;return audioStream.amplitude();}
    SystemStt(Context c,Callback callback){
        if(Build.VERSION.SDK_INT<31)throw new IllegalStateException("Android 12 이상 필요");
        context=c;this.callback=callback;
        session=new SpeechSession(new SpeechSession.Driver(){
            public void start(int token){startSegment(token);}
            public void stop(){if(audioStream!=null){audioStream.stop();return;}if(recognizer!=null)try{recognizer.stopListening();}catch(RuntimeException ignored){/* final timeout preserves partial */}}
            public void dispose(){disposeRecognizer();}
            public void later(Runnable task,long ms){main.postDelayed(()->{if(!closed)task.run();},ms);}
        },new SpeechSession.Output(){
            public void result(String text){closeResources();callback.result(text,stopped==0?0:SystemClock.elapsedRealtime()-stopped);}
            public void error(String text,String reason){closeResources();callback.error(reason);}
        },Build.VERSION.SDK_INT>=33);
    }
    void start(){lease=RecordingLease.acquire();try{session.start();}catch(RuntimeException e){cancel();throw e;}}
    void stop(){stopped=SystemClock.elapsedRealtime();session.stop();}
    String text(){return session.text();}
    void cancel(){session.cancel();closeResources();}
    private void disposeRecognizer(){
        if(audioStream!=null){audioStream.close();audioStream=null;}
        SpeechRecognizer old=recognizer;recognizer=null;
        if(old!=null){try{old.cancel();}catch(RuntimeException ignored){}try{old.destroy();}catch(RuntimeException ignored){}}
    }
    private void closeResources(){if(closed)return;closed=true;main.removeCallbacksAndMessages(null);disposeRecognizer();if(lease!=null){lease.close();lease=null;}LocalConfig.select(context,"system-session-info",(streamed()?"기본 STT · 마이크 스트림":"기본 STT · 시스템 마이크")+" · 시작 "+starts+"회 · 연속 구간 "+segments+"개 · "+(streamed()?"마이크 음량 조회 ":"음량 콜백 ")+levelEvents+"회");}
    private void startSegment(int token){
        if(Build.VERSION.SDK_INT<31){callback.error("Android 12 이상 필요");return;}
        try{
            starts++;
            recognizer=SpeechRecognizer.createOnDeviceSpeechRecognizer(context);
            recognizer.setRecognitionListener(new RecognitionListener(){
                public void onReadyForSpeech(Bundle b){}
                public void onBeginningOfSpeech(){}
                public void onRmsChanged(float rms){if(!streamed()&&session.accepts(token)){levelEvents++;callback.level(rms);}}
                public void onBufferReceived(byte[] b){}
                public void onEndOfSpeech(){
                    // In segmented mode silence ends an utterance, not the recording session.
                    if(Build.VERSION.SDK_INT>=33)return;
                    // Wait for a terminal callback before restarting, as required by Android.
                    main.postDelayed(()->{if(session.accepts(token))session.failure(token,"기본 STT 응답이 지연되어 중단했습니다. 인식된 원문은 유지됩니다.");},15000);
                }
                public void onError(int code){
                    if(!session.accepts(token))return;
                    if(code==SpeechRecognizer.ERROR_NO_MATCH||code==SpeechRecognizer.ERROR_SPEECH_TIMEOUT){session.silence(token);return;}
                    if(code==SpeechRecognizer.ERROR_LANGUAGE_NOT_SUPPORTED||code==SpeechRecognizer.ERROR_LANGUAGE_UNAVAILABLE)LocalConfig.select(context,"system-ko-state",code==SpeechRecognizer.ERROR_LANGUAGE_NOT_SUPPORTED?"unsupported":"missing");
                    session.failure(token,error(code));
                }
                public void onResults(Bundle b){session.result(token,first(b));}
                public void onSegmentResults(Bundle b){if(session.accepts(token)){segments++;session.segment(token,first(b));}}
                public void onEndOfSegmentedSession(){session.segmentedEnd(token);}
                public void onPartialResults(Bundle b){session.partial(token,first(b));}
                public void onEvent(int type,Bundle b){}
            });
            if(streamed()){
                audioStream=new SttAudioStream(lease);lease=null;recognizer.startListening(streamIntent(audioStream.input));
                audioStream.start(context,reason->main.post(()->{if(session.accepts(token))session.failure(token,reason);}));
            }else recognizer.startListening(intent());
        }catch(Exception error){session.failure(token,"기본 STT 시작 실패 · "+error.getMessage()+" · 인식된 원문은 유지됩니다.");}
    }
    private static String first(Bundle b){ArrayList<String> text=b.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION);return text==null||text.isEmpty()?"":text.get(0);}
    static String error(int code){switch(code){case SpeechRecognizer.ERROR_LANGUAGE_NOT_SUPPORTED:return "기본 STT가 한국어를 지원하지 않습니다.";case SpeechRecognizer.ERROR_LANGUAGE_UNAVAILABLE:return "한국어 음성 모델 설치가 필요합니다.";case SpeechRecognizer.ERROR_INSUFFICIENT_PERMISSIONS:return "마이크 권한이 필요합니다.";case SpeechRecognizer.ERROR_RECOGNIZER_BUSY:return "음성 인식기가 사용 중입니다. 잠시 후 다시 시도하세요.";default:return "기본 온디바이스 STT 오류 ("+code+")";}}
}
