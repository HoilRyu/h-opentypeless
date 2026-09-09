package dev.hoilryu.hopentypeless.mobile;

import android.content.*;
import android.os.*;

/** Main-thread client; each job binds a fresh, disposable inference process. */
final class LocalJob implements ServiceConnection {
    interface Callback {void stage(Bundle result);void done(Bundle result);}
    private final Context context;private final Bundle args;private final Callback callback;
    private final Handler main=new Handler(Looper.getMainLooper());
    private long started,peakPss;private int pid;
    private final Runnable sample=new Runnable(){public void run(){if(finished)return;if(pid>0){try{android.os.Debug.MemoryInfo[] info=((android.app.ActivityManager)context.getSystemService(Context.ACTIVITY_SERVICE)).getProcessMemoryInfo(new int[]{pid});if(info.length>0)peakPss=Math.max(peakPss,info[0].getTotalPss());}catch(RuntimeException ignored){}}main.postDelayed(this,1000);}};
    private boolean bound,finished;private Messenger remote;private Bundle partial=new Bundle();
    private final Runnable deadline=()->fail("로컬 처리 시간이 초과되었습니다. 더 작은 모델을 선택하세요.");
    LocalJob(Context c,Bundle args,Callback callback){this.context=c;this.args=args;this.callback=callback;partial.putString("raw",args.getString("raw",""));partial.putString("polished",args.getString("raw",""));}
    void start(){
        started=SystemClock.elapsedRealtime();
        try{bound=context.bindService(new Intent(context,LocalInferenceService.class),this,Context.BIND_AUTO_CREATE);}catch(RuntimeException error){fail("로컬 실행 서비스 연결 실패 · 다시 시도하세요.");return;}
        if(!bound)fail("로컬 실행 서비스를 시작하지 못했습니다.");else main.postDelayed(deadline,180000);
    }
    @Override public void onServiceConnected(ComponentName name,IBinder binder){
        if(finished)return;remote=new Messenger(binder);
        Messenger reply=new Messenger(new Handler(Looper.getMainLooper(),m->{
            if(finished)return true;
            if(m.what==5){pid=m.getData().getInt("pid");main.post(sample);}
            if(m.what==3){partial=new Bundle(m.getData());callback.stage(partial);}
            if(m.what==4){Bundle result=m.getData();result.putLong("peak_pss_kb",peakPss);result.putLong("elapsed_ms",SystemClock.elapsedRealtime()-started);finish();callback.done(result);}return true;
        }));
        Message message=Message.obtain(null,1);message.replyTo=reply;message.setData(args);
        try{remote.send(message);}catch(RemoteException e){fail("로컬 실행 서비스 연결 실패");}
    }
    @Override public void onServiceDisconnected(ComponentName name){if(!finished)fail("로컬 모델 실행이 중단되었습니다. 메모리 또는 모델 호환성을 확인하세요.");}
    @Override public void onBindingDied(ComponentName name){if(!finished)fail("로컬 서비스가 종료되었습니다.");}
    @Override public void onNullBinding(ComponentName name){fail("로컬 서비스 연결 실패");}
    private void fail(String error){if(finished)return;Bundle result=new Bundle(partial);result.putString("error",error);result.putString("failure_kind",ModelFailure.TRANSIENT);result.putLong("peak_pss_kb",peakPss);result.putLong("elapsed_ms",SystemClock.elapsedRealtime()-started);result.putString("failed_model",partial.containsKey("asr_ms")?args.getString("llm","none"):args.getString("stt","system"));finish();callback.done(result);}
    void cancel(){finish();}
    private void finish(){if(finished)return;finished=true;main.removeCallbacks(deadline);main.removeCallbacks(sample);
        if(remote!=null)try{remote.send(Message.obtain(null,2));}catch(RemoteException ignored){}
        if(bound){context.unbindService(this);bound=false;}
    }
}
