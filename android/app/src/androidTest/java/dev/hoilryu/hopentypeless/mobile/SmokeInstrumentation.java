package dev.hoilryu.hopentypeless.mobile;

import android.app.Instrumentation;
import android.content.*;
import android.os.*;
import android.speech.*;
import java.io.*;
import java.util.concurrent.*;

/** Explicit opt-in device smoke checks. Never records the microphone or changes network settings. */
public final class SmokeInstrumentation extends Instrumentation {
    private Bundle args;
    @Override public void onCreate(Bundle args){this.args=args;start();}
    private void report(String text){Bundle b=new Bundle();b.putString("stream",text+"\n");sendStatus(0,b);}
    @Override public void onStart(){
        Bundle result=new Bundle();
        try{
            Context c=getTargetContext();String task=args.getString("task","capabilities");
            if(task.equals("stream")||task.equals("stream-speech")){report(StreamSmoke.run(this,task.equals("stream-speech")));
            }else if(task.equals("stability")){
                File pcm=File.createTempFile("dictation-",".pcm",c.getCacheDir());
                try {
                    for(int i=0;i<100;i++){
                        try{WhisperNative.transcribe(null,pcm.getAbsolutePath(),1,new long[2]);throw new AssertionError("null model accepted");}catch(IOException expected){}
                        try{WhisperNative.transcribe("missing-model",pcm.getAbsolutePath(),1,new long[2]);throw new AssertionError("empty PCM accepted");}catch(IOException expected){}
                    }
                }finally{pcm.delete();}
                report("JNI invalid input handled 200 times without native crash");
                File wav=File.createTempFile("dictation-",".wav",c.getCacheDir());
                PcmRecorder recorder=new PcmRecorder(c,wav);
                CountDownLatch release=new CountDownLatch(1);
                Thread slow=new Thread(()->{try{release.await(5,TimeUnit.SECONDS);}catch(InterruptedException ignored){}});
                java.lang.reflect.Field thread=PcmRecorder.class.getDeclaredField("thread");thread.setAccessible(true);thread.set(recorder,slow);
                java.lang.reflect.Field owns=PcmRecorder.class.getDeclaredField("threadOwnsAudio");owns.setAccessible(true);owns.setBoolean(recorder,true);
                slow.start();
                try{
                    runOnMainSync(()->{long start=SystemClock.elapsedRealtime();recorder.cancel();if(SystemClock.elapsedRealtime()-start>500)throw new AssertionError("UI waited for recorder shutdown");});
                    if(!slow.isAlive())throw new AssertionError("delayed backend not exercised");
                }finally{release.countDown();slow.join();wav.delete();}
                report("main-thread cancel stays responsive while native cleanup is pending");
            }else if(task.equals("capabilities")){
                CountDownLatch done=new CountDownLatch(1);
                runOnMainSync(()->{
                    String reason=Capability.system(c);report("onDevice="+(reason==null?"available":reason));
                    if(reason!=null||Build.VERSION.SDK_INT<33){done.countDown();return;}
                    SpeechRecognizer r=SpeechRecognizer.createOnDeviceSpeechRecognizer(c);
                    r.checkRecognitionSupport("true".equals(args.getString("segmented"))?SystemStt.listeningIntent():SystemStt.intent(),c.getMainExecutor(),new RecognitionSupportCallback(){
                        public void onSupportResult(RecognitionSupport support){report("installed="+support.getInstalledOnDeviceLanguages()+" supported="+support.getSupportedOnDeviceLanguages()+" pending="+support.getPendingOnDeviceLanguages());r.destroy();done.countDown();}
                        public void onError(int error){report("supportError="+error);r.destroy();done.countDown();}
                    });
                });if(!done.await(20,TimeUnit.SECONDS))throw new AssertionError("support check timed out");
            }else if(task.equals("lease")){
                runOnMainSync(()->{
                    try(RecordingLease held=RecordingLease.acquire()){
                        SystemStt stt=new SystemStt(c,new SystemStt.Callback(){public void level(float v){}public void result(String s,long ms){}public void error(String e){}});
                        try{stt.start();throw new AssertionError("system STT bypassed active H recorder lease");}catch(IllegalStateException expected){}finally{stt.cancel();}
                        try(RecordingLease unexpected=RecordingLease.acquire()){throw new AssertionError("cancel released another recorder lease");}catch(IllegalStateException expected){}
                    }
                    try(RecordingLease next=RecordingLease.acquire()){report("system STT respects H recording lease without opening microphone");}
                });
            }else if(task.equals("fallback")){
                Bundle jobArgs=new Bundle();jobArgs.putString("stt","system");jobArgs.putString("llm","invalid-model");jobArgs.putString("raw","원문을 보존합니다.");
                CountDownLatch done=new CountDownLatch(1);Bundle[] output={null};runOnMainSync(()->{LocalJob job=new LocalJob(c,jobArgs,new LocalJob.Callback(){public void stage(Bundle b){}public void done(Bundle b){output[0]=b;done.countDown();}});job.start();});
                if(!done.await(20,TimeUnit.SECONDS))throw new AssertionError("job did not finish");
                if(!output[0].containsKey("error")||!"원문을 보존합니다.".equals(output[0].getString("polished")))throw new AssertionError("raw result lost on polish failure");report("raw preserved on local polish failure");
            }else if(task.equals("integrity")){
                Context sandbox=new ContextWrapper(c){@Override public File getFilesDir(){return new File(super.getFilesDir(),"transfer-test");}};
                org.json.JSONObject j=new org.json.JSONObject();j.put("id","test");j.put("label","test");j.put("file","four.bin");j.put("url","https://example.invalid/test");j.put("bytes",4);j.put("sha256","88d4266fd4e6338d13b845fcf289579d209c897823b9217da3e161936f031589");j.put("ramGb",1);j.put("availableGb",1);
                ModelCatalog.Model m=new ModelCatalog.Model(j);new ModelTransfer().importFile(sandbox,m,new ByteArrayInputStream("abcd".getBytes()),text->{});if(!m.installed(sandbox))throw new AssertionError("valid file rejected");
                for(String bad:new String[]{"abc","abcde","wxyz"}){
                    boolean rejected=false;try{new ModelTransfer().importFile(sandbox,m,new ByteArrayInputStream(bad.getBytes()),text->{});}catch(IOException expected){rejected=true;}if(!rejected)throw new AssertionError("corrupt import accepted");
                    try(InputStream in=new FileInputStream(m.path(sandbox))){if(in.read()!='a')throw new AssertionError("published file overwritten");}
                }
                m.path(sandbox).delete();m.path(sandbox).getParentFile().delete();sandbox.getFilesDir().delete();report("size/hash failures preserve installed model");
            }else if(task.equals("download")){
                ModelCatalog.Model m=ModelCatalog.get(c,args.getString("model","whisper-base"));
                new ModelTransfer().download(c,m,text->report(text));if(!m.installed(c))throw new AssertionError("not installed");report("downloadVerified="+m.id);
            }else if(task.equals("infer")||task.equals("cancel")){
                String id=args.getString("model","gemma-e2b");ModelCatalog.Model m=ModelCatalog.get(c,id);if(!m.installed(c))throw new AssertionError("model missing "+m.path(c));
                Bundle jobArgs=new Bundle();jobArgs.putString("prompt",LocalConfig.DEFAULT_PROMPT);
                if(m.whisper()){jobArgs.putString("stt",id);jobArgs.putString("llm","none");jobArgs.putString("audio",new File(c.getCacheDir(),args.getString("audio","dictation-test.wav")).getAbsolutePath());}
                else {jobArgs.putString("stt","system");jobArgs.putString("llm",id);jobArgs.putString("raw",args.getString("text","어 그럼 내일 오전 열 시에 회의를 시작하면 될 것 같아요."));}
                CountDownLatch done=new CountDownLatch(1);long start=SystemClock.elapsedRealtime();final Bundle[] output={null};
                runOnMainSync(()->{LocalJob job=new LocalJob(c,jobArgs,new LocalJob.Callback(){public void stage(Bundle b){}public void done(Bundle b){output[0]=b;done.countDown();}});job.start();if(task.equals("cancel"))new Handler(Looper.getMainLooper()).postDelayed(()->{job.cancel();done.countDown();},1000);});
                if(!done.await(195,TimeUnit.SECONDS))throw new AssertionError("inference timed out");
                if(task.equals("cancel")){report("canceled; host should check inference process exited");}
                else {report("elapsedMs="+(SystemClock.elapsedRealtime()-start)+" sttLoadMs="+output[0].getLong("stt_load_ms")+" sttInferMs="+output[0].getLong("stt_infer_ms")+" llmLoadMs="+output[0].getLong("llm_load_ms")+" llmInferMs="+output[0].getLong("llm_infer_ms")+" peakPssMB="+output[0].getLong("peak_pss_kb")/1024+" raw="+output[0].getString("raw")+" polished="+output[0].getString("polished")+" error="+output[0].getString("error"));if(output[0].containsKey("error"))throw new AssertionError(output[0].getString("error"));
                    LocalConfig.prefs(c).edit().putString("tested-"+id,"실기기 실행 확인 · "+(SystemClock.elapsedRealtime()-start)/1000.0+"초 · 품질 검증 별도").commit();}
            }else if(task.equals("ime-layout")){
                android.app.Activity activity=startActivitySync(new Intent(c,SettingsActivity.class).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK));waitForIdleSync();
                runOnMainSync(()->{android.widget.EditText input=activity.findViewById(dev.hoilryu.hopentypeless.mobile.R.id.sample_input);input.requestFocus();((android.view.inputmethod.InputMethodManager)c.getSystemService(Context.INPUT_METHOD_SERVICE)).showSoftInput(input,android.view.inputmethod.InputMethodManager.SHOW_IMPLICIT);});
                SystemClock.sleep(1500);
                android.graphics.Bitmap screen=getUiAutomation().takeScreenshot();
                if(screen==null)throw new AssertionError("no device screenshot");
                try(OutputStream out=new FileOutputStream(new File(c.getCacheDir(),"ime-layout.png"))){screen.compress(android.graphics.Bitmap.CompressFormat.PNG,100,out);}finally{screen.recycle();}
                report("captured real IME and system controls for overlap review");
            }else if(task.equals("redesign")){
                boolean saved=LocalConfig.local(c);
                try {
                    LocalConfig.local(c,true);
                    android.app.Activity local=startActivitySync(new Intent(c,SettingsActivity.class).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK));waitForIdleSync();
                    runOnMainSync(()->{String text=uiText(local.getWindow().getDecorView());if(text.contains("서버 주소")||text.contains("컴퓨터 연결"))throw new AssertionError("remote settings leaked into local home");snapshot(local.getWindow().getDecorView(),c,"redesign-local");});
                    runOnMainSync(()->button(local.getWindow().getDecorView(),"설정").performClick());waitForIdleSync();runOnMainSync(()->snapshot(local.getWindow().getDecorView(),c,"redesign-settings"));runOnMainSync(local::finish);
                    LocalConfig.local(c,false);
                    android.app.Activity remote=startActivitySync(new Intent(c,SettingsActivity.class).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK));waitForIdleSync();
                    runOnMainSync(()->{String text=uiText(remote.getWindow().getDecorView());if(!text.contains("서버 주소")||text.contains("모델 관리"))throw new AssertionError("mode separation broken");snapshot(remote.getWindow().getDecorView(),c,"redesign-remote");});runOnMainSync(remote::finish);
                    LocalConfig.local(c,true);
                    android.app.Activity models=startActivitySync(new Intent(c,LocalSettingsActivity.class).putExtra("tab","llm").addFlags(Intent.FLAG_ACTIVITY_NEW_TASK));waitForIdleSync();runOnMainSync(()->snapshot(models.getWindow().getDecorView(),c,"redesign-models"));
                    runOnMainSync(()->buttonStarting(models.getWindow().getDecorView(),"Gemma 4").performClick());waitForIdleSync();runOnMainSync(()->snapshot(models.getWindow().getDecorView(),c,"redesign-detail"));runOnMainSync(models::finish);
                    android.app.Activity preferences=startActivitySync(new Intent(c,LocalSettingsActivity.class).putExtra("tab","preferences").addFlags(Intent.FLAG_ACTIVITY_NEW_TASK));waitForIdleSync();runOnMainSync(()->snapshot(preferences.getWindow().getDecorView(),c,"redesign-preferences"));
                    runOnMainSync(()->{
                        String[] inserted={null};KeyboardPanel panel=new KeyboardPanel(preferences,new KeyboardPanel.Actions(){public void mode(){}public void record(){}public void insert(String value){inserted[0]=value;}public void clear(){}public void keyboard(){}public void settings(){}public void moveLeft(){}public void moveRight(){}public void backspace(){}});
                        int width=preferences.getWindow().getDecorView().getWidth();
                        panel.render("어 이 오류는 왜 발생하는 거야","이 오류는 왜 발생하는 거야?",false,false,false);panel.mode("로컬 · 기본 STT",false);layoutPanel(panel,width);snapshot(panel,c,"redesign-result");
                        if(android.os.Build.VERSION.SDK_INT>=30){panel.dispatchApplyWindowInsets(new android.view.WindowInsets.Builder().setInsets(android.view.WindowInsets.Type.navigationBars(),android.graphics.Insets.NONE).setInsetsIgnoringVisibility(android.view.WindowInsets.Type.navigationBars(),android.graphics.Insets.NONE).build());if(panel.getPaddingBottom()<Ui.dp(preferences,56))throw new AssertionError("zero-inset IME controls overlap toolbar");}
                        layoutPanel(panel,Ui.dp(preferences,320));assertToolbarSpacing(panel);snapshot(panel,c,"redesign-result-small");layoutPanel(panel,width);
                        button(panel,"문장 삽입").performClick();if(!"이 오류는 왜 발생하는 거야?".equals(inserted[0]))throw new AssertionError("polished insertion changed");
                        button(panel,"원문 사용").performClick();button(panel,"원문 삽입").performClick();if(!"어 이 오류는 왜 발생하는 거야".equals(inserted[0]))throw new AssertionError("raw insertion changed");
                        panel.render("","",true,false,false);panel.mode("로컬 · 기본 STT",true);panel.recordingLevel(24,6000);panel.message("중간에 쉬어도 계속 듣습니다. 끝나면 종료를 누르세요.");layoutPanel(panel,width);snapshot(panel,c,"redesign-listening");
                        if(button(panel,"종료")==null)throw new AssertionError("missing stop action");
                        panel.render("","",false,true,false);if(!button(panel,"처리 취소").isEnabled())throw new AssertionError("cancel disabled");
                    });runOnMainSync(preferences::finish);
                    report("mode separation, rendered screens, raw/polished insertion and cancel controls verified");
                }finally{LocalConfig.local(c,saved);}
            }else if(task.equals("ui")){
                Intent intent=new Intent(c,LocalSettingsActivity.class).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);android.app.Activity activity=startActivitySync(intent);
                waitForIdleSync();runOnMainSync(()->{
                    android.view.View view=activity.getWindow().getDecorView();
                    android.graphics.Bitmap bitmap=android.graphics.Bitmap.createBitmap(view.getWidth(),view.getHeight(),android.graphics.Bitmap.Config.ARGB_8888);
                    view.draw(new android.graphics.Canvas(bitmap));
                    try(OutputStream out=new FileOutputStream(new File(c.getCacheDir(),"local-ui.png"))){bitmap.compress(android.graphics.Bitmap.CompressFormat.PNG,100,out);}catch(IOException e){throw new RuntimeException(e);}finally{bitmap.recycle();}
                });runOnMainSync(()->{
                    KeyboardPanel panel=new KeyboardPanel(activity,new KeyboardPanel.Actions(){public void mode(){}public void record(){}public void insert(String s){}public void clear(){}public void keyboard(){}public void settings(){}public void moveLeft(){}public void moveRight(){}public void backspace(){}});
                    panel.render("어 그럼 내일 오전 열 시에 회의를 시작하면 될 것 같아요.","그럼 내일 오전 열 시에 회의를 시작하면 될 것 같아요.",false,false,false);panel.mode("로컬 · Whisper base",false);
                    int width=activity.getWindow().getDecorView().getWidth();panel.measure(android.view.View.MeasureSpec.makeMeasureSpec(width,android.view.View.MeasureSpec.EXACTLY),android.view.View.MeasureSpec.makeMeasureSpec(3120,android.view.View.MeasureSpec.AT_MOST));panel.layout(0,0,width,panel.getMeasuredHeight());
                    android.graphics.Bitmap image=android.graphics.Bitmap.createBitmap(width,panel.getHeight(),android.graphics.Bitmap.Config.ARGB_8888);panel.draw(new android.graphics.Canvas(image));
                    try(OutputStream out=new FileOutputStream(new File(c.getCacheDir(),"keyboard-ui.png"))){image.compress(android.graphics.Bitmap.CompressFormat.PNG,100,out);}catch(IOException e){throw new RuntimeException(e);}finally{image.recycle();}
                });report("LocalSettingsActivity and keyboard rendered");
            }else throw new AssertionError("unknown task");
            result.putString("stream","PASS "+task+"\n");finish(-1,result);
        }catch(Throwable e){result.putString("stream","FAIL "+e+"\n");finish(0,result);}
    }
    private static String uiText(android.view.View view){StringBuilder out=new StringBuilder();if(view instanceof android.widget.TextView)out.append(((android.widget.TextView)view).getText()).append('\n');if(view instanceof android.view.ViewGroup){android.view.ViewGroup group=(android.view.ViewGroup)view;for(int i=0;i<group.getChildCount();i++)out.append(uiText(group.getChildAt(i)));}return out.toString();}
    private static android.widget.Button button(android.view.View view,String label){return findButton(view,label,false);}
    private static android.widget.Button buttonStarting(android.view.View view,String label){return findButton(view,label,true);}
    private static android.widget.Button findButton(android.view.View view,String label,boolean prefix){if(view instanceof android.widget.Button){String text=((android.widget.Button)view).getText().toString();if(prefix?text.startsWith(label):text.equals(label))return (android.widget.Button)view;}if(view instanceof android.view.ViewGroup){android.view.ViewGroup group=(android.view.ViewGroup)view;for(int i=0;i<group.getChildCount();i++){android.widget.Button found=findButton(group.getChildAt(i),label,prefix);if(found!=null)return found;}}return null;}
    private static void assertToolbarSpacing(android.view.ViewGroup panel){
        android.view.ViewGroup row=(android.view.ViewGroup)panel.getChildAt(panel.getChildCount()-1);int last=-1;
        for(int i=0;i<row.getChildCount();i++){
            android.view.View child=row.getChildAt(i);if(!(child instanceof android.widget.Button)&&!(child instanceof android.widget.ImageButton))continue;
            if(child.getVisibility()!=android.view.View.VISIBLE)continue;
            int min=Ui.dp(panel.getContext(),48),gap=Ui.dp(panel.getContext(),8);
            if(child.getWidth()<min||child.getHeight()<min||child.getRight()>row.getWidth())throw new AssertionError("toolbar touch target clipped on 320dp screen");
            if(last>=0&&child.getLeft()-last<gap)throw new AssertionError("toolbar spacing too small");last=child.getRight();
        }
    }
    private static void layoutPanel(android.view.View panel,int width){panel.measure(android.view.View.MeasureSpec.makeMeasureSpec(width,android.view.View.MeasureSpec.EXACTLY),android.view.View.MeasureSpec.makeMeasureSpec(3120,android.view.View.MeasureSpec.AT_MOST));panel.layout(0,0,width,panel.getMeasuredHeight());}
    private static void snapshot(android.view.View view,Context c,String name){android.graphics.Bitmap image=android.graphics.Bitmap.createBitmap(view.getWidth(),view.getHeight(),android.graphics.Bitmap.Config.ARGB_8888);view.draw(new android.graphics.Canvas(image));try(OutputStream out=new FileOutputStream(new File(c.getCacheDir(),name+".png"))){image.compress(android.graphics.Bitmap.CompressFormat.PNG,100,out);}catch(IOException e){throw new RuntimeException(e);}finally{image.recycle();}}

}
