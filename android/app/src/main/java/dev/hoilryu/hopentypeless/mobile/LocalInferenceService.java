package dev.hoilryu.hopentypeless.mobile;

import android.app.Service;
import android.content.Intent;
import android.os.*;
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.Executors;

/** Bound, non-exported worker process. Native crashes and hard cancellation never kill the IME. */
public final class LocalInferenceService extends Service {
    private boolean busy;
    private final java.util.concurrent.ExecutorService worker=Executors.newSingleThreadExecutor();
    private final Messenger endpoint=new Messenger(new Handler(Looper.getMainLooper(),message->{
        if(message.what==2){android.os.Process.killProcess(android.os.Process.myPid());return true;}
        if(message.what!=1||busy)return true;busy=true;
        Bundle args=message.getData();Messenger reply=message.replyTo;
        Bundle identity=new Bundle();identity.putInt("pid",android.os.Process.myPid());send(reply,5,identity);
        worker.execute(()->run(args,reply));return true;
    }));
    @Override public IBinder onBind(Intent intent){return endpoint.getBinder();}
    @Override public boolean onUnbind(Intent intent){android.os.Process.killProcess(android.os.Process.myPid());return false;}
    private void send(Messenger reply,int what,Bundle data){try{Message message=Message.obtain(null,what);message.setData(data);reply.send(message);}catch(RemoteException ignored){android.os.Process.killProcess(android.os.Process.myPid());}}
    private void run(Bundle args,Messenger reply){
        Bundle result=new Bundle();String raw=args.getString("raw","");String stt=args.getString("stt","system"),llm=args.getString("llm","none");
        String stage=stt;File pcm=null;
        try {
            if(!stt.equals("system")){
                ModelCatalog.Model model=ModelCatalog.get(this,stt);if(!model.whisper()||!model.installed(this))throw new IOException("STT 모델 설치 필요");
                File audio=new File(args.getString("audio",""));
                if(!audio.getCanonicalFile().getParentFile().equals(getCacheDir().getCanonicalFile())||!audio.getName().startsWith("dictation-"))throw new IOException("잘못된 녹음 경로");
                long start=SystemClock.elapsedRealtime();pcm=File.createTempFile("dictation-",".pcm",getCacheDir());PcmAudio.decode(audio,pcm);
                long[] timings=new long[2];
                raw=new String(WhisperNative.transcribe(model.path(this).getAbsolutePath(),pcm.getAbsolutePath(),Runtime.getRuntime().availableProcessors(),timings),StandardCharsets.UTF_8).trim();
                result.putLong("stt_load_ms",timings[0]);result.putLong("stt_infer_ms",timings[1]);
                pcm.delete();pcm=null;result.putLong("asr_ms",SystemClock.elapsedRealtime()-start);
            }else result.putLong("asr_ms",args.getLong("asr_ms"));
            raw=Policy.insertion(raw);result.putString("raw",raw);result.putString("polished",raw);
            send(reply,3,new Bundle(result));
            if(!llm.equals("none")&&!raw.isEmpty()){
                stage=llm;ModelCatalog.Model model=ModelCatalog.get(this,llm);if(model.whisper()||!model.installed(this))throw new IOException("교정 모델 설치 필요");
                long start=SystemClock.elapsedRealtime();String polished=GemmaEngine.polish(model.path(this).getAbsolutePath(),getCacheDir().getAbsolutePath(),args.getString("prompt",LocalConfig.DEFAULT_PROMPT),raw,result);
                result.putString("polished",Policy.insertion(polished));result.putLong("polish_ms",SystemClock.elapsedRealtime()-start);
            }
        }catch(DictationContract.RejectedOutput error){
            // A bad completion does not mean the device cannot run this model.
            result.putString("raw",raw);result.putString("polished",raw);result.putString("error",error.getMessage());result.putBoolean("quality_rejected",true);
        }catch(Exception|LinkageError|OutOfMemoryError error){
            result.putString("raw",raw);result.putString("polished",raw);result.putString("error","로컬 처리 실패 ("+error.getClass().getSimpleName()+")");result.putString("failed_model",stage);result.putString("failure_kind",ModelFailure.classify(error));
        }finally{if(pcm!=null)pcm.delete();send(reply,4,result);}
    }
    @Override public void onDestroy(){worker.shutdownNow();super.onDestroy();}
}
