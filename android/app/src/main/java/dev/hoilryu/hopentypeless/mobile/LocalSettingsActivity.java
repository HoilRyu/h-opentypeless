package dev.hoilryu.hopentypeless.mobile;

import android.Manifest;
import android.app.Activity;
import android.content.*;
import android.content.pm.PackageManager;
import android.os.*;
import android.speech.*;
import android.widget.*;
import java.io.*;
import java.util.*;
import java.util.concurrent.*;

public final class LocalSettingsActivity extends Activity {
    private final ExecutorService worker=Executors.newSingleThreadExecutor();
    private final Handler main=new Handler(Looper.getMainLooper());
    private LinearLayout root,controls;private TextView progress,systemStatus;private ModelTransfer transfer;private LocalJob testJob;
    private ProgressBar activityProgress;
    private RadioButton systemOption;private ModelCatalog.Model importing;private SpeechRecognizer checker;private boolean rebuilding,needsRefresh;
    private static final int BG=AppUi.BG,INK=AppUi.INK,MINT=AppUi.ACCENT;
    private String tab="stt",detail;
    private final List<android.view.View> mutable=new ArrayList<>();
    private int dp(int n){return Ui.dp(this,n);}
    @Override public void onCreate(Bundle state){super.onCreate(state);if(Build.VERSION.SDK_INT>=33)getOnBackInvokedDispatcher().registerOnBackInvokedCallback(android.window.OnBackInvokedDispatcher.PRIORITY_DEFAULT,this::navigateBack);if(!LocalConfig.local(this)){startActivity(new Intent(this,SettingsActivity.class));finish();return;}tab=getIntent().getStringExtra("tab")==null?"stt":getIntent().getStringExtra("tab");if(state!=null){tab=state.getString("tab",tab);detail=state.getString("detail");}if(state!=null){String id=state.getString("importing");if(id!=null)importing=ModelCatalog.get(this,id);}build();}
    @Override public void onSaveInstanceState(Bundle state){state.putString("tab",tab);state.putString("detail",detail);if(importing!=null)state.putString("importing",importing.id);super.onSaveInstanceState(state);}
    private TextView text(LinearLayout parent,String value,int size){return AppUi.text(parent,value,size,size>=18?INK:AppUi.MUTED);}
    private Button button(LinearLayout parent,String title,Runnable action){Button b=AppUi.button(parent,title,false,action);mutable.add(b);return b;}
    private void chooseTab(String next){if(transfer!=null||testJob!=null)return;if(checker!=null){checker.destroy();checker=null;}tab=next;detail=null;build();}
    private void build(){
        rebuilding=true;mutable.clear();systemOption=null;
        ScrollView scroll=new ScrollView(this);scroll.setFillViewport(true);root=AppUi.column(this);root.setPadding(dp(20),dp(16),dp(20),dp(24));scroll.addView(root);AppUi.screen(this,scroll);
        button(root,"‹  "+(detail==null?"홈으로":"모델 관리"),()->{if(detail==null)finish();else{detail=null;build();}});
        AppUi.title(root,detail!=null?"모델 상세":tab.equals("preferences")?"내 말투 그대로.":"내 기기에 맞는 모델.");
        if(detail==null){
            LinearLayout tabs=new LinearLayout(this);root.addView(tabs);
            String[] labels={"음성 인식","문장 교정","교정 지침"},keys={"stt","llm","preferences"};
            for(int i=0;i<keys.length;i++){final String key=keys[i];LinearLayout cell=AppUi.column(this);tabs.addView(cell,AppUi.cell(this,i));Button b=button(cell,labels[i],()->chooseTab(key));AppUi.tab(b,tab.equals(key));b.setTextSize(13);b.setSelected(tab.equals(key));}
        }
        android.app.ActivityManager.MemoryInfo mem=Capability.memory(this);
        text(root,Build.MODEL+String.format(Locale.ROOT," · RAM %.1f GB · 여유 %.1f GB",mem.totalMem/1e9,mem.availMem/1e9),13);
        controls=AppUi.card(root,detail==null?(tab.equals("stt")?"음성을 글로":tab.equals("llm")?"뜻을 유지하며 다듬기":"교정 지침"):ModelCatalog.get(this,detail).label,"");
        if(detail!=null)detail(ModelCatalog.get(this,detail));
        else if(tab.equals("preferences")){
            text(controls,"질문에 답하거나 요청을 실행하지 않고 문장을 교정합니다. 아래 지침은 문체에만 적용돼요.",14);
            EditText prompt=AppUi.field(controls,"원하는 말투와 문체를 적어 주세요");prompt.setMinLines(4);prompt.setMaxLines(8);prompt.setText(LocalConfig.prompt(this));prompt.setFilters(new android.text.InputFilter[]{new android.text.InputFilter.LengthFilter(4000)});mutable.add(prompt);
            Button save=button(controls,"지침 저장",()->{LocalConfig.select(this,"prompt",prompt.getText().toString());progress.setText("교정 지침을 저장했습니다.");});AppUi.style(save,true);
            text(controls,"교정을 끄려면 문장 교정 탭에서 인식 원문 사용을 선택하세요.",13);
        }else{
            RadioGroup choices=new RadioGroup(this);controls.addView(choices);
            if(tab.equals("stt")){
                String reason=Capability.system(this);
                systemOption=radio(choices,"Android 기본 STT\n"+(reason==null?"종료 버튼까지 계속 듣기":reason),LocalConfig.stt(this).equals("system"),reason==null,()->LocalConfig.select(this,"stt","system"));
                systemStatus=text(controls,"한국어 지원 상태를 확인하고 준비할 수 있어요.",13);
                button(controls,"한국어 지원 확인",()->checkSystem(false));button(controls,"한국어 모델 준비",()->checkSystem(true));
                if(checkSelfPermission(Manifest.permission.RECORD_AUDIO)!=PackageManager.PERMISSION_GRANTED)button(controls,"마이크 권한 허용",()->requestPermissions(new String[]{Manifest.permission.RECORD_AUDIO},1));
            }else radio(choices,"교정 끄기 · 인식 원문 사용",LocalConfig.llm(this).equals("none"),true,()->LocalConfig.select(this,"llm","none"));
            for(ModelCatalog.Model m:ModelCatalog.all(this))if(m.whisper()==tab.equals("stt")){
                String selected=LocalConfig.prefs(this).getString(tab,tab.equals("stt")?"system":"none");
                Button row=AppUi.row(controls,m.whisper()?KeyIcon.Kind.MICROPHONE:KeyIcon.Kind.MODEL,m.label+(selected.equals(m.id)?" · 사용 중":""),m.size()+" · "+modelState(m),()->{detail=m.id;build();});mutable.add(row);
                if(selected.equals(m.id))row.setBackground(AppUi.shape(this,AppUi.SELECTED,14));
            }
            text(controls,tab.equals("stt")?"Whisper는 녹음 종료 후 인식해요. 같은 문장으로 기본 STT와 비교해 보세요.":"더 큰 모델이 항상 더 정확한 것은 아니에요. 설치 후 실행 상태를 확인하세요.",13);
        }
        activityProgress=new ProgressBar(this,null,android.R.attr.progressBarStyleHorizontal);activityProgress.setIndeterminate(true);activityProgress.setIndeterminateTintList(android.content.res.ColorStateList.valueOf(MINT));activityProgress.setVisibility(android.view.View.GONE);root.addView(activityProgress,new LinearLayout.LayoutParams(-1,dp(8)));
        progress=text(root,"모델은 기기 지원·설치·메모리·발열 상태에 따라 선택할 수 있습니다.",14);progress.setAccessibilityLiveRegion(android.view.View.ACCESSIBILITY_LIVE_REGION_POLITE);
        Button cancel=button(root,"진행 중 작업 취소",()->{if(transfer!=null)transfer.cancel();if(testJob!=null){testJob.cancel();testJob=null;build();progress.setText("실행 확인을 취소했습니다.");}});mutable.remove(cancel);cancel.setVisibility(transfer!=null||testJob!=null?android.view.View.VISIBLE:android.view.View.GONE);cancelAction=cancel;
        rebuilding=false;
    }
    private Button cancelAction;
    private void detail(ModelCatalog.Model m){
        text(controls,m.whisper()?"녹음한 음성을 이 기기에서 글로 바꿔요.":"화자의 뜻과 말투를 유지하며 문장을 다듬어요.",14);
        text(controls,"다운로드 크기  "+m.size()+"\n"+modelState(m),15);
        String reason=Capability.hardware(this,m);
        Button use=button(controls,"이 모델 사용",()->{String problem=Capability.ready(this,m);if(problem!=null){progress.setText(problem);return;}LocalConfig.select(this,m.whisper()?"stt":"llm",m.id);build();progress.setText("사용할 모델을 변경했습니다.");});AppUi.style(use,true);use.setEnabled(Capability.ready(this,m)==null);use.setAlpha(use.isEnabled()?1:.4f);
        Button get=button(controls,"다운로드 / 이어받기",()->download(m));get.setEnabled(reason==null&&!m.installed(this));get.setAlpha(get.isEnabled()?1:.4f);
        Button imp=button(controls,"모델 파일 가져오기",()->{importing=m;startActivityForResult(new Intent(Intent.ACTION_OPEN_DOCUMENT).setType("*/*").addCategory(Intent.CATEGORY_OPENABLE),22);});imp.setEnabled(reason==null);imp.setAlpha(imp.isEnabled()?1:.4f);
        Button test=button(controls,"실행 확인 / 재검사",()->test(m));test.setEnabled(reason==null&&m.installed(this));test.setAlpha(test.isEnabled()?1:.4f);
        Button delete=button(controls,"파일 삭제 · "+m.size(),()->new android.app.AlertDialog.Builder(this).setTitle("모델 파일을 삭제할까요?").setMessage(m.label+" · "+m.size()+"\n다시 사용하려면 설치가 필요합니다.").setNegativeButton("유지",null).setPositiveButton("삭제",(d,w)->{boolean ok=!m.path(this).exists()||m.path(this).delete();File part=new File(m.path(this)+".part");ok=(!part.exists()||part.delete())&&ok;if(ok)LocalConfig.prefs(this).edit().remove("failure-"+m.id).remove("tested-"+m.id).apply();build();progress.setText(ok?"삭제했습니다. 사용하려면 모델을 다시 준비하세요.":"파일을 삭제하지 못했습니다.");}).show());delete.setEnabled(m.installed(this)||new File(m.path(this)+".part").exists());delete.setAlpha(delete.isEnabled()?1:.4f);
        text(controls,"1  다운로드 또는 파일 가져오기\n2  파일 크기·무결성 확인\n3  실행 확인 후 선택",14);
        text(controls,"화면을 닫으면 다운로드가 중단되며 다음에 이어받을 수 있어요. 실행 확인은 정확도 검사가 아닙니다.",13);
    }
    private void navigateBack(){if(detail!=null&&transfer==null&&testJob==null){detail=null;build();}else finish();}
    // Legacy hardware/three-button back on API 28–32; newer gestures use the registered callback.
    @android.annotation.SuppressLint("GestureBackNavigation")
    @Override public void onBackPressed(){navigateBack();}
    private RadioButton radio(RadioGroup group,String title,boolean checked,boolean enabled,Runnable change){RadioButton b=new RadioButton(this);b.setId(android.view.View.generateViewId());b.setText(title);b.setTextColor(INK);b.setButtonTintList(android.content.res.ColorStateList.valueOf(MINT));group.addView(b);b.setChecked(checked);b.setEnabled(enabled);b.setAlpha(enabled?1f:.4f);mutable.add(b);b.setOnCheckedChangeListener((v,value)->{if(value&&!rebuilding){change.run();build();}});return b;}
    private void modelChoice(RadioGroup group,ModelCatalog.Model m,String key){String reason=Capability.ready(this,m);radio(group,m.label+(reason==null?"":" · "+reason),LocalConfig.prefs(this).getString(key,key.equals("stt")?"system":"none").equals(m.id),reason==null,()->LocalConfig.select(this,key,m.id));}
    private String modelState(ModelCatalog.Model m){String reason=Capability.hardware(this,m);if(reason!=null)return reason;if(!m.installed(this))return "미설치";reason=Capability.ready(this,m);if(reason!=null)return reason;return LocalConfig.prefs(this).getString("tested-"+m.id,"설치됨 · 실행 확인 전");}
    private void busy(){activityProgress.setVisibility(android.view.View.VISIBLE);if(cancelAction!=null)cancelAction.setVisibility(android.view.View.VISIBLE);for(android.view.View v:mutable){v.setEnabled(false);v.setAlpha(.4f);}progress.post(()->progress.requestRectangleOnScreen(new android.graphics.Rect(0,0,progress.getWidth(),progress.getHeight()),false));}
    private void update(String value){main.post(()->{if(!isDestroyed())progress.setText(value);});}
    private void download(ModelCatalog.Model m){
        if(transfer!=null||testJob!=null)return;transfer=new ModelTransfer();ModelTransfer current=transfer;busy();progress.setText("다운로드 연결 중…");
        worker.execute(()->{String error=null;try{current.download(this,m,this::update);}catch(Exception e){error=e.getMessage();}completeTransfer(m,error);});
    }
    private void completeTransfer(ModelCatalog.Model model,String error){main.post(()->{if(isDestroyed())return;transfer=null;if(error==null)LocalConfig.prefs(this).edit().remove("failure-"+model.id).remove("tested-"+model.id).apply();build();progress.setText(error==null?"설치 완료 · 실행 확인 후 모델을 선택하세요.":error);});}
    @Override public void onActivityResult(int request,int result,Intent data){super.onActivityResult(request,result,data);if(request!=22||result!=RESULT_OK||data==null||data.getData()==null||importing==null)return;
        ModelCatalog.Model model=importing;importing=null;android.net.Uri uri=data.getData();transfer=new ModelTransfer();ModelTransfer current=transfer;busy();
        worker.execute(()->{String error=null;try(InputStream in=getContentResolver().openInputStream(uri)){if(in==null)throw new IOException("파일을 열지 못했습니다.");current.importFile(this,model,in,this::update);}catch(Exception e){error=e.getMessage();}completeTransfer(model,error);});
    }
    private void checkSystem(boolean download){
        if(Build.VERSION.SDK_INT<33){systemStatus.setText(Capability.system(this)==null?"언어 지원 조회는 Android 13 이상에서 가능합니다. 실제 받아쓰기로 확인하세요.":Capability.system(this));return;}
        String reason=Capability.systemService(this);if(reason!=null){systemStatus.setText(reason);return;}
        if(checker!=null)checker.destroy();checker=SpeechRecognizer.createOnDeviceSpeechRecognizer(this);
        systemStatus.setText(download?"시스템에 한국어 모델 준비 요청 중…":"한국어 지원 확인 중…");
        if(download){checker.triggerModelDownload(SystemStt.intent());systemStatus.setText("한국어 모델 준비를 요청했습니다. 잠시 후 지원 확인을 다시 누르세요.");return;}
        checker.checkRecognitionSupport(SystemStt.intent(),getMainExecutor(),new RecognitionSupportCallback(){
            public void onSupportResult(RecognitionSupport support){if(isDestroyed()||systemOption==null)return;
                boolean installed=korean(support.getInstalledOnDeviceLanguages()),available=korean(support.getSupportedOnDeviceLanguages()),pending=korean(support.getPendingOnDeviceLanguages());
                String state=installed?"ready":(available||pending)?"missing":support.getSupportedOnDeviceLanguages().isEmpty()?"unknown":"unsupported";
                LocalConfig.select(LocalSettingsActivity.this,"system-ko-state",state);
                boolean enabled=Capability.system(LocalSettingsActivity.this)==null;systemOption.setEnabled(enabled);systemOption.setAlpha(enabled?1:.4f);
                systemOption.setText("Android 기본 온디바이스 STT"+(enabled?"":" · "+Capability.system(LocalSettingsActivity.this)));
                systemStatus.setText(installed?"한국어 온디바이스 모델 설치됨 · 비행기 모드 실사용 확인 필요":pending?"한국어 모델 다운로드 대기 중":available?"한국어 지원 · 모델 준비 버튼을 누르세요.":"서비스가 한국어 온디바이스 지원을 보고하지 않았습니다.");
            }
            public void onError(int error){if(!isDestroyed())systemStatus.setText("언어 지원 조회 실패 ("+error+") · 실제 받아쓰기로 확인하세요.");}
        });
    }
    static boolean korean(List<String> languages){for(String lang:languages)if(lang.equalsIgnoreCase("ko")||lang.toLowerCase(Locale.ROOT).startsWith("ko-"))return true;return false;}
    private void test(ModelCatalog.Model model){
        if(transfer!=null||testJob!=null)return;
        LocalConfig.prefs(this).edit().remove("failure-"+model.id).apply();String reason=Capability.ready(this,model);if(reason!=null){progress.setText(reason);return;}
        busy();progress.setText("모델 로딩과 실행 확인 중…");Bundle args=new Bundle();File audio;
        try {
            args.putString("prompt",LocalConfig.prompt(this));
            if(model.whisper()){
                audio=File.createTempFile("dictation-",".wav",getCacheDir());
                // Silence is only a loading smoke test, never an accuracy benchmark.
                try(InputStream in=getAssets().open("silence.wav");OutputStream out=new FileOutputStream(audio)){byte[] b=new byte[4096];int n;while((n=in.read(b))!=-1)out.write(b,0,n);}
                args.putString("stt",model.id);args.putString("llm","none");args.putString("audio",audio.getAbsolutePath());
            }else{audio=null;args.putString("stt","system");args.putString("llm",model.id);args.putString("raw","어 그럼 내일 오전 열 시에 회의를 시작하면 될 것 같아요.");}
        }catch(Exception e){build();progress.setText("검사 준비 실패");return;}
        long start=SystemClock.elapsedRealtime();
        testJob=new LocalJob(this,args,new LocalJob.Callback(){
            public void stage(Bundle result){}
            public void done(Bundle result){if(audio!=null)audio.delete();if(isDestroyed())return;testJob=null;String error=result.getString("error");
                String value=String.format(Locale.ROOT,"실행 확인 %.1f초 · 품질 검증 별도",(SystemClock.elapsedRealtime()-start)/1000.0);
                if(error==null)LocalConfig.select(LocalSettingsActivity.this,"tested-"+model.id,value);else LocalConfig.recordModelResult(LocalSettingsActivity.this,result);
                build();progress.setText(error==null?value+(model.whisper()?" · 무음 입력 검사":"\n교정 결과: "+result.getString("polished","")):error);
            }
        });testJob.start();
    }
    @Override public void onStop(){
        if(transfer!=null)transfer.cancel();
        if(testJob!=null){testJob.cancel();testJob=null;needsRefresh=true;}
        super.onStop();
    }
    @Override public void onResume(){super.onResume();if(needsRefresh){needsRefresh=false;build();}}
    @Override public void onDestroy(){if(transfer!=null)transfer.cancel();if(testJob!=null)testJob.cancel();if(checker!=null)checker.destroy();worker.shutdownNow();main.removeCallbacksAndMessages(null);super.onDestroy();}
}
